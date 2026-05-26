//! ============================================================================
//! JARVIS OS GUI — Direct pixel rendering with GPU-accelerated foundation
//! ============================================================================
//! Architecture: GPU subsystem detects hardware → Direct framebuffer UI overlay
//! The 3D renderer engine is available for GPU-backed rendering when available.
//! ============================================================================

use crate::vga_buffer::{Color, Rect, WRITER};

// ============================================================================
// DEEP DARK COLOR PALETTE
// ============================================================================
const BG_DARK: Color = Color { r: 8, g: 8, b: 14 };
const BG_PANEL: Color = Color {
    r: 12,
    g: 12,
    b: 20,
};
const BG_PANEL2: Color = Color {
    r: 10,
    g: 10,
    b: 18,
};
const BORDER_DIM: Color = Color {
    r: 18,
    g: 18,
    b: 30,
};
const BORDER_GLOW: Color = Color { r: 0, g: 60, b: 90 };
const CYAN: Color = Color {
    r: 0,
    g: 220,
    b: 255,
};
const CYAN_DIM: Color = Color {
    r: 0,
    g: 140,
    b: 200,
};
const CYAN_GLOW: Color = Color {
    r: 0,
    g: 80,
    b: 140,
};
const CYAN_BG: Color = Color { r: 0, g: 30, b: 50 };
const GREEN: Color = Color {
    r: 0,
    g: 255,
    b: 150,
};
const WHITE: Color = Color {
    r: 200,
    g: 200,
    b: 200,
};
const WHITE_DIM: Color = Color {
    r: 100,
    g: 100,
    b: 115,
};
const PURPLE: Color = Color {
    r: 160,
    g: 60,
    b: 255,
};
const YELLOW: Color = Color {
    r: 220,
    g: 200,
    b: 0,
};

// Layout
const HEADER_H: usize = 32;
const SIDEBAR_X: usize = 10;
const SIDEBAR_W: usize = 40;
const SIDEBAR_TOP: usize = 48;
const PANEL1_X: usize = 60;
const PANEL1_W: usize = 340;
const PANEL1_H: usize = 130;
const SWARM_W: usize = 500;
const SWARM_H: usize = 400;
const SWARM_TOP: usize = 48;
const LOG_H: usize = 54;
const LOG_GAP: usize = 14;
const DOCK_H: usize = 30;
const DOCK_W: usize = 540;
const ICON_SIZE: usize = 7;

// Log messages
const LOG_LINES: [&str; 8] = [
    "[SYS]  INITIALIZING SYNTHESIS SEQUENCE...",
    "[ENG]  ANALYZING SECURITY PATCH: 0x993B...",
    "[DRV]  SYNTHESIZING NEURAL DRIVER FOR NODE 04",
    "[SYS]  LOAD BALANCING CROSS-GATEWAY PROTOCOLS",
    "[NET]  SWARM SYNC COMPLETE (LATENCY 0.2ms)",
    "[EVO]  DEPLOYING AGENT_SHIELD v9.0.2",
    "[SEC]  SCANNING VIRTUAL BOUNDARIES...",
    "[NET]  PEER DISCOVERY: NODE 7 ACTIVE",
];

// ============================================================================
// DRAWING PRIMITIVES
// ============================================================================
fn draw_glow_rings(
    writer: &mut crate::vga_buffer::FramebufferWriter,
    cx: isize,
    cy: isize,
    max_r: isize,
    step: isize,
    fade_max: u8,
) {
    let mut r = max_r;
    while r > 0 {
        let t = r as f32 / max_r as f32;
        let fade = (fade_max as f32 * (1.0 - t)) as u8;
        let glow = Color {
            r: 0,
            g: fade / 2,
            b: fade,
        };
        draw_circle_px(writer, cx, cy, r, glow);
        r -= step;
    }
}

fn draw_circle_px(
    writer: &mut crate::vga_buffer::FramebufferWriter,
    cx: isize,
    cy: isize,
    r: isize,
    color: Color,
) {
    let info = writer.get_info();
    let fb_w = info.width as isize;
    let fb_h = info.height as isize;
    let mut x: isize = 0;
    let mut y: isize = r;
    let mut d = 3 - 2 * r;
    while y >= x {
        for &(dx, dy) in &[
            (x, y),
            (y, x),
            (y, -x),
            (x, -y),
            (-x, -y),
            (-y, -x),
            (-y, x),
            (-x, y),
        ] {
            let px = cx + dx;
            let py = cy + dy;
            if px >= 0 && px < fb_w && py >= 0 && py < fb_h {
                writer.write_pixel(px as usize, py as usize, color);
            }
        }
        x += 1;
        if d > 0 {
            y -= 1;
            d += 2 * (x - y) + 1;
        } else {
            d += 2 * x + 1;
        }
    }
}

fn draw_filled_circle(
    writer: &mut crate::vga_buffer::FramebufferWriter,
    cx: isize,
    cy: isize,
    r: isize,
    color: Color,
) {
    let info = writer.get_info();
    let fb_w = info.width as isize;
    let fb_h = info.height as isize;
    for dy in -r..=r {
        let half = libm::sqrtf(((r * r - dy * dy) as f32).max(0.0)) as isize;
        let x1 = (cx - half).max(0).min(fb_w - 1);
        let x2 = (cx + half).max(0).min(fb_w - 1);
        let py = cy + dy;
        if py >= 0 && py < fb_h {
            for px in x1..=x2 {
                writer.write_pixel(px as usize, py as usize, color);
            }
        }
    }
}

fn draw_arc(
    writer: &mut crate::vga_buffer::FramebufferWriter,
    cx: isize,
    cy: isize,
    r: isize,
    start_deg: f32,
    end_deg: f32,
    color: Color,
    thickness: isize,
) {
    let info = writer.get_info();
    let fb_w = info.width as isize;
    let fb_h = info.height as isize;
    for i in 0..=180 {
        let t = i as f32 / 180.0;
        let angle = start_deg + (end_deg - start_deg) * t;
        for dt in -thickness / 2..=thickness / 2 {
            let rr = (r + dt) as f32;
            let px = cx + (rr * libm::cosf(angle)) as isize;
            let py = cy + (rr * libm::sinf(angle)) as isize;
            if px >= 0 && px < fb_w && py >= 0 && py < fb_h {
                writer.write_pixel(px as usize, py as usize, color);
            }
        }
    }
}

// ============================================================================
// ICONS — 7×7 pixel art
// ============================================================================
fn draw_bitmap(
    writer: &mut crate::vga_buffer::FramebufferWriter,
    x: usize,
    y: usize,
    bmp: &[u8],
    color: Color,
) {
    for (dy, &row) in bmp.iter().enumerate() {
        for dx in 0..7 {
            if (row >> (6 - dx)) & 1 != 0 {
                writer.write_pixel(x + dx, y + dy, color);
            }
        }
    }
}

fn draw_icon(
    writer: &mut crate::vga_buffer::FramebufferWriter,
    x: usize,
    y: usize,
    t: usize,
    color: Color,
) {
    const ICONS: [&[u8]; 9] = [
        // 0: square
        &[
            0b0111110, 0b0100010, 0b0100010, 0b0100010, 0b0100010, 0b0100010, 0b0111110,
        ],
        // 1: person
        &[
            0b0011100, 0b0100010, 0b0100010, 0b0011100, 0b0001000, 0b0011100, 0b0100010,
        ],
        // 2: network
        &[
            0b1000001, 0b0100010, 0b0010100, 0b0001000, 0b0010100, 0b0100010, 0b1000001,
        ],
        // 3: shield
        &[
            0b0111110, 0b1111111, 0b1100011, 0b1100011, 0b1110111, 0b0111110, 0b0011100,
        ],
        // 4: mic
        &[
            0b0011100, 0b0100010, 0b0100010, 0b0100010, 0b0100010, 0b0100010, 0b0011100,
        ],
        // 5: logs
        &[
            0b1111110, 0b1000010, 0b1111110, 0b1000010, 0b1111110, 0b1000010, 0b1111110,
        ],
        // 6: network nav
        &[
            0b0100010, 0b1010101, 0b0100010, 0b1011101, 0b0100010, 0b1010101, 0b0100010,
        ],
        // 7: nodes
        &[
            0b0010100, 0b0100010, 0b1000001, 0b0000000, 0b1000001, 0b0100010, 0b0010100,
        ],
        // 8: security
        &[
            0b0111110, 0b1111111, 0b1111111, 0b1100011, 0b1100011, 0b0111110, 0b0011100,
        ],
    ];
    if t < ICONS.len() {
        draw_bitmap(writer, x, y, ICONS[t], color);
    }
}

// ============================================================================
// INIT
// ============================================================================
pub fn init_ui() {
    if let Some(writer) = WRITER.lock().as_mut() {
        writer.clear();
        draw_static(writer);
    }
    crate::vga_buffer::flush_fb();
    crate::serial_println!("GUI: init complete, framebuffer flushed");
}

fn draw_static(writer: &mut crate::vga_buffer::FramebufferWriter) {
    let info = writer.get_info();
    let w = info.width;
    let h = info.height;

    // Full background
    writer.fill_rect(
        Rect {
            x: 0,
            y: 0,
            width: w,
            height: h,
        },
        BG_DARK,
    );

    // === HEADER ===
    writer.fill_rect(
        Rect {
            x: 0,
            y: 0,
            width: w,
            height: HEADER_H,
        },
        BG_PANEL,
    );
    writer.fill_rect(
        Rect {
            x: 0,
            y: HEADER_H - 1,
            width: w,
            height: 1,
        },
        BORDER_GLOW,
    );
    writer.write_string_at(15, 9, "JARVIS OS", CYAN);
    writer.write_string_at(120, 13, "SYS_VER: 4.0_ALPHA", CYAN_DIM);

    // LIVE CORE ACTIVE pill
    let pill_x = w as isize - 195;
    writer.fill_rect(
        Rect {
            x: pill_x as usize,
            y: 7,
            width: 180,
            height: 18,
        },
        CYAN_BG,
    );
    writer.draw_rect(
        Rect {
            x: pill_x as usize,
            y: 7,
            width: 180,
            height: 18,
        },
        CYAN_GLOW,
    );
    draw_filled_circle(writer, pill_x + 15, 16, 3, GREEN);
    writer.write_string_at(pill_x as usize + 25, 12, "LIVE CORE ACTIVE", CYAN);

    // === LEFT SIDEBAR ===
    let ixx = SIDEBAR_X + (SIDEBAR_W - ICON_SIZE) / 2;
    writer.fill_rect(
        Rect {
            x: SIDEBAR_X,
            y: SIDEBAR_TOP,
            width: SIDEBAR_W,
            height: 295,
        },
        BG_PANEL,
    );
    writer.draw_rect(
        Rect {
            x: SIDEBAR_X,
            y: SIDEBAR_TOP,
            width: SIDEBAR_W,
            height: 295,
        },
        BORDER_DIM,
    );
    draw_icon(writer, ixx, SIDEBAR_TOP + 15, 0, CYAN);
    draw_icon(writer, ixx, SIDEBAR_TOP + 85, 1, GREEN);
    draw_icon(writer, ixx, SIDEBAR_TOP + 155, 2, CYAN);
    draw_icon(writer, ixx, SIDEBAR_TOP + 225, 3, PURPLE);

    // === SUBSTRATE HEALTH PANEL ===
    writer.fill_rect(
        Rect {
            x: PANEL1_X,
            y: SIDEBAR_TOP,
            width: PANEL1_W,
            height: PANEL1_H,
        },
        BG_PANEL,
    );
    writer.draw_rect(
        Rect {
            x: PANEL1_X,
            y: SIDEBAR_TOP,
            width: PANEL1_W,
            height: PANEL1_H,
        },
        BORDER_DIM,
    );
    writer.write_string_at(PANEL1_X + 12, SIDEBAR_TOP + 8, "SUBSTRATE HEALTH", CYAN_DIM);

    // === SWARM VISUALIZER ===
    let sw_x = w - SWARM_W - 10;
    writer.fill_rect(
        Rect {
            x: sw_x,
            y: SWARM_TOP,
            width: SWARM_W,
            height: SWARM_H,
        },
        BG_PANEL,
    );
    writer.draw_rect(
        Rect {
            x: sw_x,
            y: SWARM_TOP,
            width: SWARM_W,
            height: SWARM_H,
        },
        BORDER_DIM,
    );
    writer.write_string_at(sw_x + 12, SWARM_TOP + 8, "SWARM VISUALIZER", CYAN_DIM);
    writer.write_string_at(sw_x + SWARM_W - 55, SWARM_TOP + 8, "LIVE", GREEN);

    // === LOG WINDOW ===
    let log_y = h - LOG_H - DOCK_H - 12;
    writer.fill_rect(
        Rect {
            x: 10,
            y: log_y,
            width: w - 20,
            height: LOG_H,
        },
        BG_PANEL,
    );
    writer.draw_rect(
        Rect {
            x: 10,
            y: log_y,
            width: w - 20,
            height: LOG_H,
        },
        BORDER_DIM,
    );

    // === NAV DOCK ===
    let dock_y = h - DOCK_H - 6;
    let center_x = w / 2;
    writer.fill_rect(
        Rect {
            x: center_x - DOCK_W / 2,
            y: dock_y,
            width: DOCK_W,
            height: DOCK_H,
        },
        BG_PANEL,
    );
    writer.draw_rect(
        Rect {
            x: center_x - DOCK_W / 2,
            y: dock_y,
            width: DOCK_W,
            height: DOCK_H,
        },
        BORDER_GLOW,
    );

    let nav_start = center_x - DOCK_W / 2 + 15;
    let nav_item_w = (DOCK_W - 30) / 5;
    let labels = ["VOICE", "LOGS", "NETWORK", "NODES", "SECURITY"];
    let icons_idx = [4, 5, 6, 7, 8]; // mic, logs, network_nav, nodes, security
    let colors = [CYAN, WHITE_DIM, WHITE_DIM, WHITE_DIM, WHITE_DIM];

    for i in 0..5 {
        let nx = nav_start + i * nav_item_w;
        if i == 0 {
            writer.fill_rect(
                Rect {
                    x: nx - 5,
                    y: dock_y + 2,
                    width: nav_item_w + 10,
                    height: DOCK_H - 4,
                },
                CYAN_BG,
            );
        }
        let ix = nx + (nav_item_w - ICON_SIZE) / 2;
        draw_icon(writer, ix, dock_y + 4, icons_idx[i], colors[i]);
        writer.write_string_at(
            nx + (nav_item_w - labels[i].len() * 8) / 2,
            dock_y + DOCK_H - 10,
            labels[i],
            colors[i],
        );
    }

    crate::serial_println!("GUI: Layout @ {}x{}", w, h);
}

// ============================================================================
// UI TASK
// ============================================================================
pub async fn ui_task() {
    let mut angle: f32 = 0.0;
    let mut tick: u64 = 0;
    loop {
        update(angle, tick);

        // Draw a bright pulsing debug marker to verify the loop runs
        if let Some(writer) = WRITER.lock().as_mut() {
            let info = writer.get_info();
            let w = info.width;
            let h = info.height;
            // Pulsing white dot at center
            let pulse = (libm::sinf(angle * 3.0) * 0.5 + 0.5) * 255.0;
            let px = w / 2;
            let py = h / 2 + 160;
            for dy in -5..5 {
                for dx in -5..5 {
                    writer.write_pixel(
                        px + dx as usize,
                        py + dy as usize,
                        Color {
                            r: pulse as u8,
                            g: pulse as u8,
                            b: pulse as u8,
                        },
                    );
                }
            }
        }

        angle += 0.06;
        tick += 1;
        if angle > core::f32::consts::TAU {
            angle = 0.0;
        }
        // Flush framebuffer cache to make writes visible
        crate::vga_buffer::flush_fb();
        for _ in 0..2 {
            crate::task::yield_now().await;
        }
    }
}

fn update(angle: f32, tick: u64) {
    if let Some(writer) = WRITER.lock().as_mut() {
        let info = writer.get_info();
        let w = info.width;
        let h = info.height;

        // === 1. CPU GAUGE ===
        let gc_x = PANEL1_X + 75;
        let gc_y = SIDEBAR_TOP + 78;
        let gr = 30;
        let cpu = 45 + ((tick.wrapping_mul(7) ^ (tick >> 2)) % 20) as u8;

        writer.fill_rect(
            Rect {
                x: PANEL1_X + 5,
                y: SIDEBAR_TOP + 30,
                width: PANEL1_W - 10,
                height: PANEL1_H - 38,
            },
            BG_PANEL,
        );
        writer.write_string_at(PANEL1_X + 120, SIDEBAR_TOP + 35, "CPU LOAD", WHITE_DIM);
        writer.write_string_at(PANEL1_X + 120, SIDEBAR_TOP + 55, "OPTIMAL RANGE", WHITE_DIM);

        let fill = cpu as f32 / 100.0;
        let sa = -2.35619;
        let tot = 4.71239;
        draw_arc(
            writer,
            gc_x as isize,
            gc_y as isize,
            gr,
            sa,
            sa + tot,
            BORDER_DIM,
            6,
        );
        let ac = if cpu < 70 {
            GREEN
        } else if cpu < 90 {
            YELLOW
        } else {
            Color {
                r: 255,
                g: 50,
                b: 50,
            }
        };
        draw_arc(
            writer,
            gc_x as isize,
            gc_y as isize,
            gr,
            sa,
            sa + tot * fill,
            ac,
            6,
        );
        draw_filled_circle(writer, gc_x as isize, gc_y as isize, gr - 5, BG_PANEL);

        let pct = alloc::format!("{:02}%", cpu);
        writer.write_string_at(gc_x - 16, gc_y - 5, &pct, WHITE);

        let bx = PANEL1_X + 120;
        let by = SIDEBAR_TOP + 80;
        writer.fill_rect(
            Rect {
                x: bx,
                y: by,
                width: 195,
                height: 8,
            },
            BORDER_DIM,
        );
        let fw = ((cpu as usize).min(100) * 195) / 100;
        writer.fill_rect(
            Rect {
                x: bx,
                y: by,
                width: fw,
                height: 8,
            },
            ac,
        );

        // === 2. CENTER CORE ===
        let ccx = ((w as isize / 2) - 30);
        let ccy: isize = 360;
        let cr: isize = 130;
        let cb = 2 * cr + 80;

        writer.fill_rect(
            Rect {
                x: (ccx - cr - 10) as usize,
                y: (ccy - cr - 10) as usize,
                width: cb as usize,
                height: cb as usize + 30,
            },
            BG_DARK,
        );

        let breath = 2.0 + libm::sinf(angle * 1.5) * 1.5;
        draw_glow_rings(writer, ccx, ccy, cr + 50, 4, 6);
        draw_glow_rings(writer, ccx, ccy, cr + 20, 3, 12);
        draw_glow_rings(writer, ccx, ccy, cr + 5, 2, 20);
        draw_glow_rings(writer, ccx, ccy, cr - 10, 4, 8);

        draw_circle_px(writer, ccx, ccy, cr + breath as isize + 5, CYAN);
        draw_circle_px(writer, ccx, ccy, cr + breath as isize + 7, CYAN_GLOW);
        draw_circle_px(writer, ccx, ccy, cr, CYAN_DIM);

        // Mic icon
        draw_icon(writer, (ccx - 10) as usize, (ccy - 12) as usize, 4, CYAN);

        // CORE_ACTIVE text
        let cl = "CORE_ACTIVE";
        writer.write_string_at(
            (ccx - (cl.len() as isize * 4) - 10) as usize,
            (ccy + cr + 12) as usize,
            cl,
            CYAN,
        );

        // Sine waveform
        let wy = (ccy + cr + 35) as usize;
        let wa = 6.0 + libm::sinf(angle * 0.5) * 3.0;
        let wf = 3.0 + libm::sinf(angle * 0.3) * 0.5;
        writer.fill_rect(
            Rect {
                x: (ccx - 120) as usize,
                y: wy - 10,
                width: 240,
                height: 20,
            },
            BG_DARK,
        );

        for x in 0..200 {
            let wx = (ccx - 100 + x as isize) as usize;
            let wave = libm::sinf(angle * wf + x as f32 * 0.08) * wa;
            let ws = (wy as f32 + wave) as usize;
            if ws < h && ws > 0 {
                writer.write_pixel(wx, ws, CYAN);
                if ws + 1 < h {
                    writer.write_pixel(wx, ws + 1, CYAN_DIM);
                }
                if ws > 0 {
                    writer.write_pixel(wx, ws - 1, CYAN_DIM);
                }
            }
        }

        // === 3. SWARM VISUALIZER ===
        let sw_x = w - SWARM_W - 10;
        writer.fill_rect(
            Rect {
                x: sw_x + 3,
                y: SWARM_TOP + 20,
                width: SWARM_W - 6,
                height: SWARM_H - 23,
            },
            BG_PANEL,
        );

        let hub_x = sw_x + SWARM_W / 2;
        let hub_y = SWARM_TOP + SWARM_H / 2;
        let mut positions = alloc::vec::Vec::new();

        for i in 0..9 {
            let na = angle * 0.25 + (i as f32 * core::f32::consts::TAU / 9.0);
            let nd = 90.0 + libm::sinf(angle * 0.4 + i as f32 * 1.3) * 40.0;
            let nx = (hub_x as isize + (nd * libm::cosf(na)) as isize)
                .max(sw_x as isize + 10)
                .min(sw_x as isize + SWARM_W as isize - 10) as usize;
            let ny = (hub_y as isize + (nd * libm::sinf(na)) as isize)
                .max(SWARM_TOP as isize + 25)
                .min(SWARM_TOP as isize + SWARM_H as isize - 10) as usize;
            positions.push((nx, ny));

            writer.draw_line(
                hub_x as isize,
                hub_y as isize,
                nx as isize,
                ny as isize,
                Color {
                    r: 0,
                    g: 60,
                    b: 100,
                },
            );

            for j in 0..i {
                let (ox, oy) = positions[j];
                let dx = (nx as isize - ox as isize).abs();
                let dy = (ny as isize - oy as isize).abs();
                if dx < 200 && dy < 200 {
                    let dist = libm::sqrtf((dx * dx + dy * dy) as f32);
                    if dist < 250.0 {
                        let fade = (200.0 - dist * 0.8).max(30.0) as u8;
                        writer.draw_line(
                            nx as isize,
                            ny as isize,
                            ox as isize,
                            oy as isize,
                            Color {
                                r: 0,
                                g: fade / 3,
                                b: fade / 2,
                            },
                        );
                    }
                }
            }

            let nr = (3.0 + libm::sinf(angle + i as f32 * 2.0) * 1.5 + 2.0) as isize;
            draw_filled_circle(
                writer,
                nx as isize,
                ny as isize,
                nr + 2,
                Color { r: 0, g: 30, b: 50 },
            );
            draw_filled_circle(writer, nx as isize, ny as isize, nr, CYAN);
        }

        let hr = (6.0 + libm::sinf(angle * 1.8) * 3.0) as isize;
        draw_filled_circle(
            writer,
            hub_x as isize,
            hub_y as isize,
            hr + 4,
            Color { r: 0, g: 40, b: 70 },
        );
        draw_filled_circle(writer, hub_x as isize, hub_y as isize, hr + 2, CYAN_GLOW);
        draw_filled_circle(writer, hub_x as isize, hub_y as isize, hr, WHITE);

        // === 4. LOG WINDOW ===
        let log_y = h - LOG_H - DOCK_H - 12;
        writer.fill_rect(
            Rect {
                x: 13,
                y: log_y + 3,
                width: w - 26,
                height: LOG_H - 6,
            },
            BG_PANEL2,
        );

        let mi = (tick as usize / 15) % LOG_LINES.len();
        for li in 0..3 {
            let idx = (mi + li) % LOG_LINES.len();
            let msg = LOG_LINES[idx];
            let lc = if msg.starts_with("[NET]") || msg.starts_with("[SYS]") {
                CYAN
            } else if msg.starts_with("[EVO]") {
                GREEN
            } else if msg.starts_with("[SEC]") {
                YELLOW
            } else {
                CYAN_DIM
            };
            writer.fill_rect(
                Rect {
                    x: 16,
                    y: log_y + 5 + li * LOG_GAP,
                    width: w - 32,
                    height: 12,
                },
                BG_PANEL2,
            );
            writer.write_string_at(18, log_y + 5 + li * LOG_GAP, msg, lc);
        }

        // === 5. NAV DOCK PULSE ===
        let dock_y = h - DOCK_H - 6;
        let center_x = w / 2;
        let ps = (2.0 + libm::sinf(angle * 2.5) * 1.5 + 1.5) as isize;
        let vx = center_x - DOCK_W / 2 + 15 + (DOCK_W - 30) / 10;
        draw_filled_circle(writer, vx as isize + 15, dock_y as isize + 10, ps, GREEN);
    }
}
