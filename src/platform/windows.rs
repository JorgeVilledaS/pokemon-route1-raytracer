use crate::core::{Camera, Vec3};
use crate::materials::MaterialLibrary;
use crate::render::{CpuRenderer, SceneMode};
use crate::scene::adventure::{overview_target, start_position, Adventure};
use crate::scene::Scene;
use std::ffi::c_void;
use std::io;
use std::mem::size_of;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::{Duration, Instant};

const RENDER_WIDTH: usize = 480;
const RENDER_HEIGHT: usize = 270;
const WINDOW_WIDTH: i32 = 960;
const WINDOW_HEIGHT: i32 = 540;

const WS_WINDOWED_FIXED: u32 = 0x00CA_0000;
const CW_USEDEFAULT: i32 = 0x8000_0000_u32 as i32;
const SW_SHOW: i32 = 5;
const PM_REMOVE: u32 = 0x0001;
const DIB_RGB_COLORS: u32 = 0;
const SRCCOPY: u32 = 0x00CC_0020;
const BI_RGB: u32 = 0;

const WM_DESTROY: u32 = 0x0002;
const WM_QUIT: u32 = 0x0012;
const WM_MOUSEWHEEL: u32 = 0x020A;

const VK_ESCAPE: i32 = 0x1B;
const VK_LEFT: i32 = 0x25;
const VK_UP: i32 = 0x26;
const VK_RIGHT: i32 = 0x27;
const VK_DOWN: i32 = 0x28;
const VK_LBUTTON: i32 = 0x01;
const VK_A: i32 = 0x41;
const VK_D: i32 = 0x44;
const VK_N: i32 = 0x4E;
const VK_R: i32 = 0x52;
const VK_S: i32 = 0x53;
const VK_W: i32 = 0x57;
const IDC_ARROW: usize = 32_512;

static WHEEL_DELTA: AtomicI32 = AtomicI32::new(0);

type Handle = *mut c_void;
type WindowProc = unsafe extern "system" fn(Handle, u32, usize, isize) -> isize;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct Message {
    hwnd: Handle,
    message: u32,
    w_param: usize,
    l_param: isize,
    time: u32,
    point: Point,
    private: u32,
}

#[repr(C)]
struct WindowClass {
    style: u32,
    window_proc: Option<WindowProc>,
    class_extra: i32,
    window_extra: i32,
    instance: Handle,
    icon: Handle,
    cursor: Handle,
    background: Handle,
    menu_name: *const u16,
    class_name: *const u16,
}

#[repr(C)]
struct BitmapInfoHeader {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bit_count: u16,
    compression: u32,
    size_image: u32,
    x_pixels_per_meter: i32,
    y_pixels_per_meter: i32,
    colors_used: u32,
    colors_important: u32,
}

#[repr(C)]
struct BitmapInfo {
    header: BitmapInfoHeader,
    colors: [u32; 1],
}

#[link(name = "user32")]
extern "system" {
    fn RegisterClassW(class: *const WindowClass) -> u16;
    fn CreateWindowExW(
        extended_style: u32,
        class_name: *const u16,
        window_name: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: Handle,
        menu: Handle,
        instance: Handle,
        parameter: *mut c_void,
    ) -> Handle;
    fn DefWindowProcW(hwnd: Handle, message: u32, w_param: usize, l_param: isize) -> isize;
    fn ShowWindow(hwnd: Handle, command: i32) -> i32;
    fn UpdateWindow(hwnd: Handle) -> i32;
    fn PeekMessageW(
        message: *mut Message,
        hwnd: Handle,
        minimum: u32,
        maximum: u32,
        remove: u32,
    ) -> i32;
    fn TranslateMessage(message: *const Message) -> i32;
    fn DispatchMessageW(message: *const Message) -> isize;
    fn PostQuitMessage(exit_code: i32);
    fn LoadCursorW(instance: Handle, cursor_name: *const u16) -> Handle;
    fn GetAsyncKeyState(key: i32) -> i16;
    fn GetForegroundWindow() -> Handle;
    fn GetCursorPos(point: *mut Point) -> i32;
    fn ScreenToClient(hwnd: Handle, point: *mut Point) -> i32;
    fn GetClientRect(hwnd: Handle, rect: *mut Rect) -> i32;
    fn AdjustWindowRect(rect: *mut Rect, style: u32, has_menu: i32) -> i32;
    fn GetDC(hwnd: Handle) -> Handle;
    fn ReleaseDC(hwnd: Handle, device_context: Handle) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(module_name: *const u16) -> Handle;
}

#[link(name = "gdi32")]
extern "system" {
    fn StretchDIBits(
        device_context: Handle,
        destination_x: i32,
        destination_y: i32,
        destination_width: i32,
        destination_height: i32,
        source_x: i32,
        source_y: i32,
        source_width: i32,
        source_height: i32,
        bits: *const c_void,
        bitmap_info: *const BitmapInfo,
        usage: u32,
        raster_operation: u32,
    ) -> i32;
    fn TextOutW(dc: Handle, x: i32, y: i32, text: *const u16, count: i32) -> i32;
    fn SetTextColor(dc: Handle, color: u32) -> u32;
    fn SetBkColor(dc: Handle, color: u32) -> u32;
    fn SetBkMode(dc: Handle, mode: i32) -> i32;
}

pub fn run(mut scene: Scene, _target: Vec3, materials: MaterialLibrary) -> io::Result<()> {
    let window = create_window()?;
    let renderer = CpuRenderer::new(RENDER_WIDTH, RENDER_HEIGHT);
    println!(
        "CPU renderer: {} workers, up to {} scanlines per worker ({}x{} internal)",
        renderer.worker_count(),
        renderer.rows_per_worker(),
        renderer.width(),
        renderer.height()
    );
    println!("Controls: WASD walk | SPACE auto | TAB overview | mouse/arrows orbit | wheel/Q/E zoom | N atmosphere | P restart");

    let mut controls = CameraControls::new();
    let mut game = Adventure::new(42);
    let mut auto_down = false;
    let mut overview = true;
    let mut overview_down = false;
    let mut restart_down = false;
    game.sync(&mut scene);
    let target = start_position();
    let mut previous_time = Instant::now();
    let mut frame = renderer.render(&scene, &controls.camera(target), &materials, controls.mode);
    let mut rendered_frames = 0_usize;

    loop {
        if !pump_messages() {
            break;
        }
        if unsafe { GetForegroundWindow() } != window {
            previous_time = Instant::now();
            std::thread::sleep(Duration::from_millis(25));
            continue;
        }
        if key_down(VK_ESCAPE) {
            break;
        }

        let now = Instant::now();
        let delta_seconds = (now - previous_time).as_secs_f64().min(0.05);
        previous_time = now;
        controls.update(window, delta_seconds);
        for index in 1..=3 {
            if key_down(0x30 + index)
                && game
                    .encounters
                    .iter()
                    .any(|e| e.captured && e.skin == index as usize)
            {
                game.skin = index as usize;
            }
        }
        let auto = key_down(0x20);
        if auto && !auto_down {
            game.auto = !game.auto;
        }
        auto_down = auto;
        let tab = key_down(0x09);
        if tab && !overview_down {
            overview = !overview;
        }
        overview_down = tab;
        let restart = key_down(0x50);
        if restart && !restart_down {
            game = Adventure::new(42);
            game.sync(&mut scene);
        }
        restart_down = restart;
        {
            let forward = Vec3::new(-controls.yaw.sin(), 0.0, -controls.yaw.cos());
            let right = Vec3::new(controls.yaw.cos(), 0.0, -controls.yaw.sin());
            let input = forward
                * (f64::from(key_down(VK_W) as u8) - f64::from(key_down(VK_S) as u8))
                + right * (f64::from(key_down(VK_D) as u8) - f64::from(key_down(VK_A) as u8));
            game.update(&scene, delta_seconds, input);
            game.sync(&mut scene);
        }
        let target = if overview {
            overview_target()
        } else {
            game.position + Vec3::new(0.0, 0.5, -3.5)
        };
        let mut camera = controls.camera(target);
        if overview {
            camera.distance = 88.0;
            camera.pitch = 58.0_f64.to_radians();
        }

        {
            let stats = renderer.render_into(
                &mut frame.pixels,
                &scene,
                &camera,
                &materials,
                controls.mode,
            );
            frame.elapsed = stats.elapsed;
            frame.total_dda_steps = stats.total_dda_steps;
            rendered_frames += 1;

            if rendered_frames.is_multiple_of(30) {
                println!(
                    "frame {rendered_frames}: {:.1} ms, {:.1} average DDA steps/ray",
                    frame.elapsed.as_secs_f64() * 1_000.0,
                    frame.total_dda_steps as f64 / (RENDER_WIDTH * RENDER_HEIGHT) as f64
                );
            }
        }

        present(window, &frame.pixels)?;
        draw_hud(window, &game, controls.mode);
        std::thread::sleep(Duration::from_millis(8));
    }

    Ok(())
}

struct CameraControls {
    yaw: f64,
    pitch: f64,
    distance: f64,
    previous_mouse: Option<Point>,
    reset_was_down: bool,
    night_was_down: bool,
    mode: SceneMode,
}

impl CameraControls {
    fn new() -> Self {
        Self {
            yaw: 0.0,
            pitch: 30.0_f64.to_radians(),
            distance: 14.0,
            previous_mouse: None,
            reset_was_down: false,
            night_was_down: false,
            mode: SceneMode::Day,
        }
    }

    fn camera(&self, target: Vec3) -> Camera {
        Camera::new(
            target,
            self.yaw,
            self.pitch,
            self.distance,
            60.0,
            RENDER_WIDTH as f64 / RENDER_HEIGHT as f64,
        )
    }

    fn update(&mut self, window: Handle, delta_seconds: f64) -> bool {
        let mut changed = false;
        let orbit_delta = 1.8 * delta_seconds;
        let zoom_delta = 7.0 * delta_seconds;

        if key_down(VK_LEFT) {
            self.yaw -= orbit_delta;
            changed = true;
        }
        if key_down(VK_RIGHT) {
            self.yaw += orbit_delta;
            changed = true;
        }
        if key_down(VK_UP) {
            self.pitch += orbit_delta;
            changed = true;
        }
        if key_down(VK_DOWN) {
            self.pitch -= orbit_delta;
            changed = true;
        }
        if key_down(0x51) {
            self.distance -= zoom_delta;
            changed = true;
        }
        if key_down(0x45) {
            self.distance += zoom_delta;
            changed = true;
        }

        let wheel = WHEEL_DELTA.swap(0, Ordering::Relaxed);
        if wheel != 0 {
            self.distance -= f64::from(wheel) / 120.0;
            changed = true;
        }

        if key_down(VK_LBUTTON) {
            if let Some(mouse) = cursor_in_client(window) {
                if let Some(previous) = self.previous_mouse {
                    self.yaw -= f64::from(mouse.x - previous.x) * 0.008;
                    self.pitch -= f64::from(mouse.y - previous.y) * 0.008;
                    changed |= mouse.x != previous.x || mouse.y != previous.y;
                }
                self.previous_mouse = Some(mouse);
            }
        } else {
            self.previous_mouse = None;
        }

        let reset_is_down = key_down(VK_R);
        if reset_is_down && !self.reset_was_down {
            let reset = Self::new();
            self.yaw = reset.yaw;
            self.pitch = reset.pitch;
            self.distance = reset.distance;
            changed = true;
        }
        self.reset_was_down = reset_is_down;

        let night_is_down = key_down(VK_N);
        if night_is_down && !self.night_was_down {
            self.mode = match self.mode {
                SceneMode::Day => SceneMode::Night,
                SceneMode::Night => SceneMode::Day,
            };
            println!("Scene mode: {:?}", self.mode);
            changed = true;
        }
        self.night_was_down = night_is_down;

        self.yaw = self.yaw.rem_euclid(std::f64::consts::TAU);
        self.pitch = self
            .pitch
            .clamp(-89.0_f64.to_radians(), 89.0_f64.to_radians());
        self.distance = self.distance.clamp(3.0, 40.0);
        changed
    }
}

fn create_window() -> io::Result<Handle> {
    let class_name = wide_string("PokemonRoute1RaytracerWindow");
    let title = wide_string("Pokemon Route 1 - CPU Raytracer");

    unsafe {
        let instance = GetModuleHandleW(null());
        let class = WindowClass {
            style: 0,
            window_proc: Some(window_proc),
            class_extra: 0,
            window_extra: 0,
            instance,
            icon: null_mut(),
            cursor: LoadCursorW(null_mut(), IDC_ARROW as *const u16),
            background: null_mut(),
            menu_name: null(),
            class_name: class_name.as_ptr(),
        };

        if RegisterClassW(&class) == 0 {
            return Err(io::Error::last_os_error());
        }

        let mut window_rect = Rect {
            left: 0,
            top: 0,
            right: WINDOW_WIDTH,
            bottom: WINDOW_HEIGHT,
        };
        if AdjustWindowRect(&mut window_rect, WS_WINDOWED_FIXED, 0) == 0 {
            return Err(io::Error::last_os_error());
        }

        let window = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_WINDOWED_FIXED,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            window_rect.right - window_rect.left,
            window_rect.bottom - window_rect.top,
            null_mut(),
            null_mut(),
            instance,
            null_mut(),
        );
        if window.is_null() {
            return Err(io::Error::last_os_error());
        }

        ShowWindow(window, SW_SHOW);
        UpdateWindow(window);
        Ok(window)
    }
}

fn pump_messages() -> bool {
    unsafe {
        let mut message = Message {
            hwnd: null_mut(),
            message: 0,
            w_param: 0,
            l_param: 0,
            time: 0,
            point: Point::default(),
            private: 0,
        };

        while PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) != 0 {
            if message.message == WM_QUIT {
                return false;
            }
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    true
}

fn present(window: Handle, pixels: &[u32]) -> io::Result<()> {
    let bitmap_info = BitmapInfo {
        header: BitmapInfoHeader {
            size: size_of::<BitmapInfoHeader>() as u32,
            width: RENDER_WIDTH as i32,
            height: -(RENDER_HEIGHT as i32),
            planes: 1,
            bit_count: 32,
            compression: BI_RGB,
            size_image: 0,
            x_pixels_per_meter: 0,
            y_pixels_per_meter: 0,
            colors_used: 0,
            colors_important: 0,
        },
        colors: [0],
    };

    unsafe {
        let mut client = Rect::default();
        if GetClientRect(window, &mut client) == 0 {
            return Err(io::Error::last_os_error());
        }
        let device_context = GetDC(window);
        if device_context.is_null() {
            return Err(io::Error::last_os_error());
        }

        StretchDIBits(
            device_context,
            0,
            0,
            client.right - client.left,
            client.bottom - client.top,
            0,
            0,
            RENDER_WIDTH as i32,
            RENDER_HEIGHT as i32,
            pixels.as_ptr().cast(),
            &bitmap_info,
            DIB_RGB_COLORS,
            SRCCOPY,
        );
        ReleaseDC(window, device_context);
    }

    Ok(())
}

fn cursor_in_client(window: Handle) -> Option<Point> {
    unsafe {
        let mut point = Point::default();
        (GetCursorPos(&mut point) != 0 && ScreenToClient(window, &mut point) != 0).then_some(point)
    }
}

fn draw_hud(window: Handle, game: &Adventure, mode: SceneMode) {
    unsafe {
        let dc = GetDC(window);
        if dc.is_null() {
            return;
        }
        SetBkMode(dc, 2);
        SetBkColor(dc, 0x002c2018);
        SetTextColor(dc, 0x00e8ffff);
        let lines = [
            format!(
                "RUTA 01 / CAPTURAS {}/{} / {:?}",
                game.count(),
                game.encounters.len(),
                mode
            ),
            "WASD caminar | ESPACIO auto | TAB mapa | raton/flechas orbita | rueda/Q/E zoom"
                .to_string(),
            "N dia/noche | 1/2/3 apariencias capturadas | P reiniciar".to_string(),
            if game.finished {
                "LLEGASTE AL FINAL! P para volver a explorar".to_string()
            } else {
                format!(
                    "{} | Recorrido {:.0}%",
                    if game.auto {
                        "PASEO AUTOMATICO"
                    } else {
                        "EXPLORACION"
                    },
                    game.progress() * 100.0
                )
            },
        ];
        for (i, line) in lines.iter().enumerate() {
            let w: Vec<u16> = line.encode_utf16().collect();
            TextOutW(dc, 18, 14 + i as i32 * 24, w.as_ptr(), w.len() as i32);
        }
        ReleaseDC(window, dc);
    }
}

fn key_down(key: i32) -> bool {
    unsafe { GetAsyncKeyState(key) as u16 & 0x8000 != 0 }
}

fn wide_string(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

unsafe extern "system" fn window_proc(
    window: Handle,
    message: u32,
    w_param: usize,
    l_param: isize,
) -> isize {
    match message {
        WM_MOUSEWHEEL => {
            let delta = ((w_param >> 16) as u16) as i16;
            WHEEL_DELTA.fetch_add(i32::from(delta), Ordering::Relaxed);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(window, message, w_param, l_param),
    }
}
