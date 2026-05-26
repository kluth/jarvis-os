//! ============================================================================
//! JARVIS OS GPU Subsystem — Universal Graphics Driver Database
//! ============================================================================
//! Supports 200+ GPU families from all known manufacturers:
//!   3dfx, Nvidia, ATI/AMD, Intel, S3, Matrox, SiS, Rendition,
//!   PowerVR, Number Nine, Trident, XGI, Realtek, ARM Mali,
//!   Qualcomm Adreno, Broadcom VideoCore, Cirrus Logic, AST,
//!   VMware, VirtIO, VirtualBox, Hyper-V, Mediatek, Rockchip, Allwinner
//! ============================================================================

use crate::vga_buffer::{Color, Rect};
use alloc::vec::Vec;

pub mod mmio;
pub mod drivers;

// ============================================================================
// KNOWN GPU FAMILIES — 200+ variants across all manufacturers
// ============================================================================
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpuFamily {
    // ---- 3dfx Interactive (6) ----
    Voodoo1, Voodoo2, VoodooRush, VoodooBanshee, Voodoo3, Voodoo4, Voodoo5,

    // ---- Nvidia (60+) ----
    Nv1, Nv2, Nv3, Nv4, Nv5,
    Riva128, Riva128ZX, RivaTNT, RivaTNT2,
    GeForce256, GeForceDDR,
    GeForce2MX, GeForce2GTS, GeForce2Ultra, GeForce2Go, GeForce2,
    GeForce3, GeForce3Ti,
    GeForce4MX, GeForce4Ti, GeForce4Go, GeForce4,
    GeForceFX5200, GeForceFX5600, GeForceFX5700, GeForceFX5800,
    GeForceFX5900, GeForceFX5950, GeForceFXGo, GeForceFX,
    GeForce6,
    GeForce6200, GeForce6600, GeForce6800, GeForce6Go,
    GeForce7,
    GeForce7300, GeForce7400, GeForce7500, GeForce7600, GeForce7800, GeForce7900, GeForce7950,
    GeForce8,
    GeForce8300, GeForce8400, GeForce8500, GeForce8600, GeForce8800,
    GeForce9,
    GeForce9600, GeForce9800,
    GeForceGTX200,
    GTX260, GTX280, GTX285, GTX295,
    GeForceGTX400,
    GTX460, GTX465, GTX470, GTX480,
    GeForceGTX500,
    GTX550, GTX560, GTX570, GTX580, GTX590,
    GeForceGTX600,
    GTX650, GTX660, GTX670, GTX680, GTX690,
    GeForceGTX700,
    GTX760, GTX770, GTX780, GTX780Ti, GTXTitan,
    GeForceGTX900,
    GTX960, GTX970, GTX980, GTX980Ti, GTXTitanX,
    GeForceGTX10,
    GTX1050, GTX1060, GTX1070, GTX1080, GTX1080Ti, GTXTitanXP,
    GeForceGTX16,
    GTX1650, GTX1660, GTX1660Super, GTX1660Ti,
    GeForceRTX20,
    RTX2060, RTX2070, RTX2080, RTX2080Ti, TitanRTX,
    GeForceRTX30,
    RTX3060, RTX3070, RTX3080, RTX3090, RTX3090Ti,
    GeForceRTX40,
    RTX4060, RTX4070, RTX4080, RTX4090,
    GeForceRTX50,
    RTX5060, RTX5070, RTX5080, RTX5090,
    NvidiaQuadro, NvidiaTesla,

    // ---- ATI / AMD (60+) ----
    RagePro, RageXL, RageFury, Rage128VR, Rage128GL, Rage128Pro, Rage128,
    Radeon7000, Radeon7200, Radeon7500, Radeon8500, Radeon8500LE,
    Radeon9000, Radeon9100, Radeon9200, Radeon9500, Radeon9550, Radeon9600,
    Radeon9700, Radeon9700Pro, Radeon9800, Radeon9800Pro,
    RadeonX300, RadeonX600, RadeonX700, RadeonX800, RadeonX850,
    RadeonX1300, RadeonX1600, RadeonX1800, RadeonX1900, RadeonX1950,
    RadeonHD2400, RadeonHD2600, RadeonHD2900,
    RadeonHD3450, RadeonHD3650, RadeonHD3850, RadeonHD3870,
    RadeonHD4350, RadeonHD4550, RadeonHD4650, RadeonHD4670,
    RadeonHD4770, RadeonHD4830, RadeonHD4850, RadeonHD4870, RadeonHD4870X2,
    RadeonHD5450, RadeonHD5570, RadeonHD5670, RadeonHD5750, RadeonHD5770,
    RadeonHD5830, RadeonHD5850, RadeonHD5870, RadeonHD5970,
    RadeonHD6450, RadeonHD6570, RadeonHD6670, RadeonHD6750, RadeonHD6770,
    RadeonHD6790, RadeonHD6850, RadeonHD6870, RadeonHD6950, RadeonHD6970, RadeonHD6990,
    RadeonHD7750, RadeonHD7770, RadeonHD7850, RadeonHD7870, RadeonHD7950, RadeonHD7970, RadeonHD7990,
    RadeonR7240, RadeonR7250, RadeonR7260, RadeonR7270, RadeonR7280,
    RadeonR9280, RadeonR9290, RadeonR9290X, RadeonR9390, RadeonR9390X,
    RadeonR7460, RadeonR7470, RadeonR7480, RadeonR7570, RadeonR7580, RadeonR7590,
    RadeonRX460, RadeonRX470, RadeonRX480, RadeonRX550, RadeonRX560,
    RadeonRX570, RadeonRX580, RadeonRX590,
    RadeonRX5500, RadeonRX5600, RadeonRX5700, RadeonRX6400, RadeonRX6500XT,
    RadeonRX6600, RadeonRX6700XT, RadeonRX6800, RadeonRX6800XT, RadeonRX6900XT,
    RadeonRX7600, RadeonRX7700XT, RadeonRX7800XT, RadeonRX7900GRE, RadeonRX7900XT, RadeonRX7900XTX,
    RadeonRX9070, RadeonRX9070XT,
    AMDFirePro, AMDInstinct,

    // ---- Intel Integrated (15+) ----
    Intel740, Intel810, Intel915,
    IntelGMA3000, IntelGMA3100, IntelGMAX3100, IntelGMAX3500,
    IntelHDGraphics, IntelHDGraphics2000, IntelHDGraphics2500, IntelHDGraphics3000,
    IntelHDGraphics4000, IntelHDGraphics4200, IntelHDGraphics4400, IntelHDGraphics4600,
    IntelHDGraphics5000, IntelHDGraphics5100, IntelHDGraphics5200,
    IntelHDGraphics5300, IntelHDGraphics5500, IntelHDGraphics6000,
    IntelHDGraphics6100, IntelHDGraphics615, IntelHDGraphics620, IntelHDGraphics630,
    IntelHDGraphics640, IntelHDGraphics650, IntelHDGraphicsP630,
    IntelIrisPlus640, IntelIrisPlus645, IntelIrisPlus650, IntelIrisPro580,
    IntelIrisXe, IntelIrisXeMax,
    IntelArcA310, IntelArcA380, IntelArcA580, IntelArcA750, IntelArcA770,
    IntelArcB580, IntelArcB770,
    IntelXeLLVM,

    // ---- S3 Graphics (10) ----
    S3Trio64, S3Virge, S3Savage3D, S3Savage4, S3Savage2000,
    S3ChromeS20, S3ChromeS25, S3ChromeS27, S3DeltaChrome, S3GammaChrome,

    // ---- Matrox (10) ----
    MatroxMGA, MatroxG100, MatroxG200, MatroxG400, MatroxG450, MatroxG550,
    MatroxParhelia, MatroxP650, MatroxP690, MatroxM9148,

    // ---- SiS (10) ----
    SiS300, SiS315, SiS330, SiS340,
    SiSXabre200, SiSXabre400, SiSXabre600,
    SiSMirage, SiSMirage2, SiSMirage3, SiSM771,

    // ---- Rendition (3) ----
    RenditionV1000, RenditionV2000, RenditionV2200,

    // ---- PowerVR / Imagination (8) ----
    PowerVRPCX1, PowerVRPCX2, PowerVRKyro, PowerVRKyro2,
    PowerVRMBX, PowerVR5SGX, PowerVR6Rogue, PowerVR7XT,

    // ---- Number Nine (2) ----
    NumberNineImagine128, NumberNineRevolution4,

    // ---- Trident (8) ----
    TridentProVidia9680, Trident3DImage9750, Trident3DImage9850,
    TridentCyberBlade, TridentCyberBladeXP, TridentBlade3D, TridentXP4,

    // ---- XGI (5) ----
    XGIVolariV3, XGIVolariV5, XGIVolariV8, XGIVolariZ7, XGIVolariZ9,

    // ---- Realtek (2) ----
    RealtekRV280, RealtekRV380,

    // ---- ARM Mali (10) ----
    ARMMali400, ARMMali450, ARMMaliT604, ARMMaliT628, ARMMaliT760, ARMMaliT860,
    ARMMaliG71, ARMMaliG76, ARMMaliG78, ARMMaliG310,

    // ---- Qualcomm Adreno (15) ----
    Adreno200, Adreno205, Adreno220, Adreno225, Adreno302, Adreno305,
    Adreno320, Adreno330, Adreno405, Adreno418, Adreno420, Adreno430,
    Adreno504, Adreno505, Adreno506, Adreno508, Adreno509, Adreno510, Adreno512,
    Adreno530, Adreno540,
    Adreno615, Adreno616, Adreno618, Adreno619, Adreno620,
    Adreno630, Adreno640, Adreno650, Adreno660, Adreno670, Adreno680,
    Adreno730, Adreno740, Adreno750,

    // ---- Broadcom VideoCore (3) ----
    VideoCoreIV, VideoCoreV, VideoCoreVI,

    // ---- Mediatek (6) ----
    MediatekMaliG52, MediatekMaliG57, MediatekMaliG68,
    MediatekMaliG77, MediatekMaliG610, MediatekMaliG615,

    // ---- Rockchip / Allwinner (4) ----
    RockchipMali400, RockchipMaliT760,
    AllwinnerMali400, RockchipMaliT860,

    // ---- AST / ASpeed (6) ----
    AST2000, AST2100, AST2300, AST2400, AST2500, AST2600,

    // ---- Cirrus Logic (6) ----
    CirrusLogic5430, CirrusLogic5446, CirrusLogic5464, CirrusLogic5465,
    CirrusLogic67200, CirrusLogic7548,

    // ---- Hermes / Virtual (8) ----
    VMWareSVGA, VMWareSVGA3, VirtIOGPU, VirtIOGPU3D,
    BochsVBE, QXL, VirtualBoxVGA, HyperVSynthetic,

    // ---- Legacy / Generic (2) ----
    StandardVGA, VesaVBE,
    Unknown,
}

// ============================================================================
// GPU NAME DATABASE
// ============================================================================
pub fn gpu_name(family: GpuFamily) -> &'static str {
    match family {
        GpuFamily::Voodoo1        => "3dfx Voodoo Graphics",
        GpuFamily::Voodoo2        => "3dfx Voodoo2",
        GpuFamily::VoodooRush     => "3dfx Voodoo Rush",
        GpuFamily::VoodooBanshee  => "3dfx Voodoo Banshee",
        GpuFamily::Voodoo3        => "3dfx Voodoo3",
        GpuFamily::Voodoo4        => "3dfx Voodoo4 4500",
        GpuFamily::Voodoo5        => "3dfx Voodoo5 5500",

        GpuFamily::Nv1            => "Nvidia NV1",
        GpuFamily::Nv2            => "Nvidia NV2",
        GpuFamily::Nv3            => "Nvidia NV3 (RIVA 128)",
        GpuFamily::Nv4            => "Nvidia NV4 (RIVA TNT)",
        GpuFamily::Nv5            => "Nvidia NV5 (RIVA TNT2)",
        GpuFamily::Riva128        => "Nvidia RIVA 128",
        GpuFamily::Riva128ZX      => "Nvidia RIVA 128 ZX",
        GpuFamily::RivaTNT        => "Nvidia RIVA TNT",
        GpuFamily::RivaTNT2       => "Nvidia RIVA TNT2",

        GpuFamily::GeForce256     => "Nvidia GeForce 256",
        GpuFamily::GeForceDDR     => "Nvidia GeForce 256 DDR",
        GpuFamily::GeForce2MX     => "Nvidia GeForce2 MX",
        GpuFamily::GeForce2GTS    => "Nvidia GeForce2 GTS",
        GpuFamily::GeForce2Ultra  => "Nvidia GeForce2 Ultra",
        GpuFamily::GeForce2Go     => "Nvidia GeForce2 Go",
        GpuFamily::GeForce2       => "Nvidia GeForce2",
        GpuFamily::GeForce3       => "Nvidia GeForce3",
        GpuFamily::GeForce3Ti     => "Nvidia GeForce3 Ti",
        GpuFamily::GeForce4MX     => "Nvidia GeForce4 MX",
        GpuFamily::GeForce4Ti     => "Nvidia GeForce4 Ti",
        GpuFamily::GeForce4Go     => "Nvidia GeForce4 Go",
        GpuFamily::GeForce4       => "Nvidia GeForce4",

        GpuFamily::GeForceFX5200  => "Nvidia GeForce FX 5200",
        GpuFamily::GeForceFX5600  => "Nvidia GeForce FX 5600",
        GpuFamily::GeForceFX5700  => "Nvidia GeForce FX 5700",
        GpuFamily::GeForceFX5800  => "Nvidia GeForce FX 5800",
        GpuFamily::GeForceFX5900  => "Nvidia GeForce FX 5900",
        GpuFamily::GeForceFX5950  => "Nvidia GeForce FX 5950",
        GpuFamily::GeForceFXGo    => "Nvidia GeForce FX Go",
        GpuFamily::GeForceFX      => "Nvidia GeForce FX",

        GpuFamily::GeForce6       => "Nvidia GeForce 6",
        GpuFamily::GeForce6200    => "Nvidia GeForce 6200",
        GpuFamily::GeForce6600    => "Nvidia GeForce 6600",
        GpuFamily::GeForce6800    => "Nvidia GeForce 6800",
        GpuFamily::GeForce6Go     => "Nvidia GeForce Go 6",

        GpuFamily::GeForce7       => "Nvidia GeForce 7",
        GpuFamily::GeForce7300    => "Nvidia GeForce 7300",
        GpuFamily::GeForce7400    => "Nvidia GeForce 7400",
        GpuFamily::GeForce7500    => "Nvidia GeForce 7500",
        GpuFamily::GeForce7600    => "Nvidia GeForce 7600",
        GpuFamily::GeForce7800    => "Nvidia GeForce 7800",
        GpuFamily::GeForce7900    => "Nvidia GeForce 7900",
        GpuFamily::GeForce7950    => "Nvidia GeForce 7950 GX2",

        GpuFamily::GeForce8       => "Nvidia GeForce 8",
        GpuFamily::GeForce8300    => "Nvidia GeForce 8300",
        GpuFamily::GeForce8400    => "Nvidia GeForce 8400",
        GpuFamily::GeForce8500    => "Nvidia GeForce 8500",
        GpuFamily::GeForce8600    => "Nvidia GeForce 8600",
        GpuFamily::GeForce8800    => "Nvidia GeForce 8800",

        GpuFamily::GeForce9       => "Nvidia GeForce 9",
        GpuFamily::GeForce9600    => "Nvidia GeForce 9600",
        GpuFamily::GeForce9800    => "Nvidia GeForce 9800",

        GpuFamily::GeForceGTX200  => "Nvidia GeForce GTX 200",
        GpuFamily::GTX260         => "Nvidia GeForce GTX 260",
        GpuFamily::GTX280         => "Nvidia GeForce GTX 280",
        GpuFamily::GTX285         => "Nvidia GeForce GTX 285",
        GpuFamily::GTX295         => "Nvidia GeForce GTX 295",

        GpuFamily::GeForceGTX400  => "Nvidia GeForce GTX 400",
        GpuFamily::GTX460         => "Nvidia GeForce GTX 460",
        GpuFamily::GTX465         => "Nvidia GeForce GTX 465",
        GpuFamily::GTX470         => "Nvidia GeForce GTX 470",
        GpuFamily::GTX480         => "Nvidia GeForce GTX 480",

        GpuFamily::GeForceGTX500  => "Nvidia GeForce GTX 500",
        GpuFamily::GTX550         => "Nvidia GeForce GTX 550 Ti",
        GpuFamily::GTX560         => "Nvidia GeForce GTX 560",
        GpuFamily::GTX570         => "Nvidia GeForce GTX 570",
        GpuFamily::GTX580         => "Nvidia GeForce GTX 580",
        GpuFamily::GTX590         => "Nvidia GeForce GTX 590",

        GpuFamily::GeForceGTX600  => "Nvidia GeForce GTX 600 (Kepler)",
        GpuFamily::GTX650         => "Nvidia GeForce GTX 650",
        GpuFamily::GTX660         => "Nvidia GeForce GTX 660",
        GpuFamily::GTX670         => "Nvidia GeForce GTX 670",
        GpuFamily::GTX680         => "Nvidia GeForce GTX 680",
        GpuFamily::GTX690         => "Nvidia GeForce GTX 690",

        GpuFamily::GeForceGTX700  => "Nvidia GeForce GTX 700",
        GpuFamily::GTX760         => "Nvidia GeForce GTX 760",
        GpuFamily::GTX770         => "Nvidia GeForce GTX 770",
        GpuFamily::GTX780         => "Nvidia GeForce GTX 780",
        GpuFamily::GTX780Ti       => "Nvidia GeForce GTX 780 Ti",
        GpuFamily::GTXTitan       => "Nvidia GeForce GTX Titan",

        GpuFamily::GeForceGTX900  => "Nvidia GeForce GTX 900 (Maxwell)",
        GpuFamily::GTX960         => "Nvidia GeForce GTX 960",
        GpuFamily::GTX970         => "Nvidia GeForce GTX 970",
        GpuFamily::GTX980         => "Nvidia GeForce GTX 980",
        GpuFamily::GTX980Ti       => "Nvidia GeForce GTX 980 Ti",
        GpuFamily::GTXTitanX      => "Nvidia GeForce GTX Titan X",

        GpuFamily::GeForceGTX10   => "Nvidia GeForce GTX 10 (Pascal)",
        GpuFamily::GTX1050        => "Nvidia GeForce GTX 1050",
        GpuFamily::GTX1060        => "Nvidia GeForce GTX 1060",
        GpuFamily::GTX1070        => "Nvidia GeForce GTX 1070",
        GpuFamily::GTX1080        => "Nvidia GeForce GTX 1080",
        GpuFamily::GTX1080Ti      => "Nvidia GeForce GTX 1080 Ti",
        GpuFamily::GTXTitanXP     => "Nvidia Titan Xp",

        GpuFamily::GeForceGTX16   => "Nvidia GeForce GTX 16 (Turing)",
        GpuFamily::GTX1650        => "Nvidia GeForce GTX 1650",
        GpuFamily::GTX1660        => "Nvidia GeForce GTX 1660",
        GpuFamily::GTX1660Super   => "Nvidia GeForce GTX 1660 Super",
        GpuFamily::GTX1660Ti      => "Nvidia GeForce GTX 1660 Ti",

        GpuFamily::GeForceRTX20   => "Nvidia GeForce RTX 20 (Turing)",
        GpuFamily::RTX2060        => "Nvidia GeForce RTX 2060",
        GpuFamily::RTX2070        => "Nvidia GeForce RTX 2070",
        GpuFamily::RTX2080        => "Nvidia GeForce RTX 2080",
        GpuFamily::RTX2080Ti      => "Nvidia GeForce RTX 2080 Ti",
        GpuFamily::TitanRTX       => "Nvidia Titan RTX",

        GpuFamily::GeForceRTX30   => "Nvidia GeForce RTX 30 (Ampere)",
        GpuFamily::RTX3060        => "Nvidia GeForce RTX 3060",
        GpuFamily::RTX3070        => "Nvidia GeForce RTX 3070",
        GpuFamily::RTX3080        => "Nvidia GeForce RTX 3080",
        GpuFamily::RTX3090        => "Nvidia GeForce RTX 3090",
        GpuFamily::RTX3090Ti      => "Nvidia GeForce RTX 3090 Ti",

        GpuFamily::GeForceRTX40   => "Nvidia GeForce RTX 40 (Ada Lovelace)",
        GpuFamily::RTX4060        => "Nvidia GeForce RTX 4060",
        GpuFamily::RTX4070        => "Nvidia GeForce RTX 4070",
        GpuFamily::RTX4080        => "Nvidia GeForce RTX 4080",
        GpuFamily::RTX4090        => "Nvidia GeForce RTX 4090",

        GpuFamily::GeForceRTX50   => "Nvidia GeForce RTX 50 (Blackwell)",
        GpuFamily::RTX5060        => "Nvidia GeForce RTX 5060",
        GpuFamily::RTX5070        => "Nvidia GeForce RTX 5070",
        GpuFamily::RTX5080        => "Nvidia GeForce RTX 5080",
        GpuFamily::RTX5090        => "Nvidia GeForce RTX 5090",

        GpuFamily::NvidiaQuadro   => "Nvidia Quadro",
        GpuFamily::NvidiaTesla    => "Nvidia Tesla",

        // ATI / AMD
        GpuFamily::RagePro        => "ATI 3D Rage Pro",
        GpuFamily::RageXL         => "ATI Rage XL",
        GpuFamily::RageFury       => "ATI Rage Fury",
        GpuFamily::Rage128VR      => "ATI Rage 128 VR",
        GpuFamily::Rage128GL      => "ATI Rage 128 GL",
        GpuFamily::Rage128Pro     => "ATI Rage 128 Pro",
        GpuFamily::Rage128        => "ATI Rage 128",
        GpuFamily::Radeon7000     => "ATI Radeon 7000 (R100)",
        GpuFamily::Radeon7200     => "ATI Radeon 7200",
        GpuFamily::Radeon7500     => "ATI Radeon 7500 (RV200)",
        GpuFamily::Radeon8500     => "ATI Radeon 8500 (R200)",
        GpuFamily::Radeon8500LE   => "ATI Radeon 8500 LE",
        GpuFamily::Radeon9000     => "ATI Radeon 9000",
        GpuFamily::Radeon9100     => "ATI Radeon 9100",
        GpuFamily::Radeon9200     => "ATI Radeon 9200",
        GpuFamily::Radeon9500     => "ATI Radeon 9500",
        GpuFamily::Radeon9550     => "ATI Radeon 9550",
        GpuFamily::Radeon9600     => "ATI Radeon 9600",
        GpuFamily::Radeon9700     => "ATI Radeon 9700 (R300)",
        GpuFamily::Radeon9700Pro  => "ATI Radeon 9700 Pro",
        GpuFamily::Radeon9800     => "ATI Radeon 9800",
        GpuFamily::Radeon9800Pro  => "ATI Radeon 9800 Pro",

        GpuFamily::RadeonX300     => "ATI Radeon X300",
        GpuFamily::RadeonX600     => "ATI Radeon X600",
        GpuFamily::RadeonX700     => "ATI Radeon X700",
        GpuFamily::RadeonX800     => "ATI Radeon X800",
        GpuFamily::RadeonX850     => "ATI Radeon X850",
        GpuFamily::RadeonX1300    => "ATI Radeon X1300",
        GpuFamily::RadeonX1600    => "ATI Radeon X1600",
        GpuFamily::RadeonX1800    => "ATI Radeon X1800",
        GpuFamily::RadeonX1900    => "ATI Radeon X1900",
        GpuFamily::RadeonX1950    => "ATI Radeon X1950",

        GpuFamily::RadeonHD2400   => "ATI Radeon HD 2400",
        GpuFamily::RadeonHD2600   => "ATI Radeon HD 2600",
        GpuFamily::RadeonHD2900   => "ATI Radeon HD 2900",
        GpuFamily::RadeonHD3450   => "ATI Radeon HD 3450",
        GpuFamily::RadeonHD3650   => "ATI Radeon HD 3650",
        GpuFamily::RadeonHD3850   => "ATI Radeon HD 3850",
        GpuFamily::RadeonHD3870   => "ATI Radeon HD 3870",
        GpuFamily::RadeonHD4350   => "ATI Radeon HD 4350",
        GpuFamily::RadeonHD4550   => "ATI Radeon HD 4550",
        GpuFamily::RadeonHD4650   => "ATI Radeon HD 4650",
        GpuFamily::RadeonHD4670   => "ATI Radeon HD 4670",
        GpuFamily::RadeonHD4770   => "ATI Radeon HD 4770",
        GpuFamily::RadeonHD4830   => "ATI Radeon HD 4830",
        GpuFamily::RadeonHD4850   => "ATI Radeon HD 4850",
        GpuFamily::RadeonHD4870   => "ATI Radeon HD 4870",
        GpuFamily::RadeonHD4870X2 => "ATI Radeon HD 4870 X2",
        GpuFamily::RadeonHD5450   => "ATI Radeon HD 5450",
        GpuFamily::RadeonHD5570   => "ATI Radeon HD 5570",
        GpuFamily::RadeonHD5670   => "ATI Radeon HD 5670",
        GpuFamily::RadeonHD5750   => "ATI Radeon HD 5750",
        GpuFamily::RadeonHD5770   => "ATI Radeon HD 5770",
        GpuFamily::RadeonHD5830   => "ATI Radeon HD 5830",
        GpuFamily::RadeonHD5850   => "ATI Radeon HD 5850",
        GpuFamily::RadeonHD5870   => "ATI Radeon HD 5870",
        GpuFamily::RadeonHD5970   => "ATI Radeon HD 5970",
        GpuFamily::RadeonHD6450   => "AMD Radeon HD 6450",
        GpuFamily::RadeonHD6570   => "AMD Radeon HD 6570",
        GpuFamily::RadeonHD6670   => "AMD Radeon HD 6670",
        GpuFamily::RadeonHD6750   => "AMD Radeon HD 6750",
        GpuFamily::RadeonHD6770   => "AMD Radeon HD 6770",
        GpuFamily::RadeonHD6790   => "AMD Radeon HD 6790",
        GpuFamily::RadeonHD6850   => "AMD Radeon HD 6850",
        GpuFamily::RadeonHD6870   => "AMD Radeon HD 6870",
        GpuFamily::RadeonHD6950   => "AMD Radeon HD 6950",
        GpuFamily::RadeonHD6970   => "AMD Radeon HD 6970",
        GpuFamily::RadeonHD6990   => "AMD Radeon HD 6990",
        GpuFamily::RadeonHD7750   => "AMD Radeon HD 7750",
        GpuFamily::RadeonHD7770   => "AMD Radeon HD 7770",
        GpuFamily::RadeonHD7850   => "AMD Radeon HD 7850",
        GpuFamily::RadeonHD7870   => "AMD Radeon HD 7870",
        GpuFamily::RadeonHD7950   => "AMD Radeon HD 7950",
        GpuFamily::RadeonHD7970   => "AMD Radeon HD 7970",
        GpuFamily::RadeonHD7990   => "AMD Radeon HD 7990",

        GpuFamily::RadeonR7240    => "AMD Radeon R7 240",
        GpuFamily::RadeonR7250    => "AMD Radeon R7 250",
        GpuFamily::RadeonR7260    => "AMD Radeon R7 260",
        GpuFamily::RadeonR7270    => "AMD Radeon R7 270",
        GpuFamily::RadeonR7280    => "AMD Radeon R7 280",
        GpuFamily::RadeonR9280    => "AMD Radeon R9 280",
        GpuFamily::RadeonR9290    => "AMD Radeon R9 290",
        GpuFamily::RadeonR9290X   => "AMD Radeon R9 290X",
        GpuFamily::RadeonR9390    => "AMD Radeon R9 390",
        GpuFamily::RadeonR9390X   => "AMD Radeon R9 390X",
        GpuFamily::RadeonR7460    => "AMD Radeon R7 460",
        GpuFamily::RadeonR7470    => "AMD Radeon R7 470",
        GpuFamily::RadeonR7480    => "AMD Radeon R7 480",
        GpuFamily::RadeonR7570    => "AMD Radeon R7 570",
        GpuFamily::RadeonR7580    => "AMD Radeon R7 580",
        GpuFamily::RadeonR7590    => "AMD Radeon R7 590",

        GpuFamily::RadeonRX460    => "AMD Radeon RX 460",
        GpuFamily::RadeonRX470    => "AMD Radeon RX 470",
        GpuFamily::RadeonRX480    => "AMD Radeon RX 480",
        GpuFamily::RadeonRX550    => "AMD Radeon RX 550",
        GpuFamily::RadeonRX560    => "AMD Radeon RX 560",
        GpuFamily::RadeonRX570    => "AMD Radeon RX 570",
        GpuFamily::RadeonRX580    => "AMD Radeon RX 580",
        GpuFamily::RadeonRX590    => "AMD Radeon RX 590",
        GpuFamily::RadeonRX5500   => "AMD Radeon RX 5500",
        GpuFamily::RadeonRX5600   => "AMD Radeon RX 5600",
        GpuFamily::RadeonRX5700   => "AMD Radeon RX 5700",
        GpuFamily::RadeonRX6400   => "AMD Radeon RX 6400",
        GpuFamily::RadeonRX6500XT => "AMD Radeon RX 6500 XT",
        GpuFamily::RadeonRX6600   => "AMD Radeon RX 6600",
        GpuFamily::RadeonRX6700XT => "AMD Radeon RX 6700 XT",
        GpuFamily::RadeonRX6800   => "AMD Radeon RX 6800",
        GpuFamily::RadeonRX6800XT => "AMD Radeon RX 6800 XT",
        GpuFamily::RadeonRX6900XT => "AMD Radeon RX 6900 XT",
        GpuFamily::RadeonRX7600   => "AMD Radeon RX 7600",
        GpuFamily::RadeonRX7700XT => "AMD Radeon RX 7700 XT",
        GpuFamily::RadeonRX7800XT => "AMD Radeon RX 7800 XT",
        GpuFamily::RadeonRX7900GRE => "AMD Radeon RX 7900 GRE",
        GpuFamily::RadeonRX7900XT => "AMD Radeon RX 7900 XT",
        GpuFamily::RadeonRX7900XTX => "AMD Radeon RX 7900 XTX",
        GpuFamily::RadeonRX9070   => "AMD Radeon RX 9070",
        GpuFamily::RadeonRX9070XT => "AMD Radeon RX 9070 XT",
        GpuFamily::AMDFirePro     => "AMD FirePro",
        GpuFamily::AMDInstinct    => "AMD Instinct",

        // Intel Integrated
        GpuFamily::Intel740       => "Intel i740",
        GpuFamily::Intel810       => "Intel i810",
        GpuFamily::Intel915       => "Intel GMA 915/950",
        GpuFamily::IntelGMA3000   => "Intel GMA 3000",
        GpuFamily::IntelGMA3100   => "Intel GMA 3100",
        GpuFamily::IntelGMAX3100  => "Intel GMA X3100",
        GpuFamily::IntelGMAX3500  => "Intel GMA X3500",
        GpuFamily::IntelHDGraphics => "Intel HD Graphics",
        GpuFamily::IntelHDGraphics2000 => "Intel HD Graphics 2000",
        GpuFamily::IntelHDGraphics2500 => "Intel HD Graphics 2500",
        GpuFamily::IntelHDGraphics3000 => "Intel HD Graphics 3000",
        GpuFamily::IntelHDGraphics4000 => "Intel HD Graphics 4000",
        GpuFamily::IntelHDGraphics4200 => "Intel HD Graphics 4200",
        GpuFamily::IntelHDGraphics4400 => "Intel HD Graphics 4400",
        GpuFamily::IntelHDGraphics4600 => "Intel HD Graphics 4600",
        GpuFamily::IntelHDGraphics5000 => "Intel HD Graphics 5000",
        GpuFamily::IntelHDGraphics5100 => "Intel HD Graphics 5100",
        GpuFamily::IntelHDGraphics5200 => "Intel HD Graphics 5200 (Iris Pro)",
        GpuFamily::IntelHDGraphics5300 => "Intel HD Graphics 5300",
        GpuFamily::IntelHDGraphics5500 => "Intel HD Graphics 5500",
        GpuFamily::IntelHDGraphics6000 => "Intel HD Graphics 6000",
        GpuFamily::IntelHDGraphics6100 => "Intel HD Graphics 610",
        GpuFamily::IntelHDGraphics615 => "Intel HD Graphics 615",
        GpuFamily::IntelHDGraphics620 => "Intel HD Graphics 620",
        GpuFamily::IntelHDGraphics630 => "Intel HD Graphics 630",
        GpuFamily::IntelHDGraphics640 => "Intel HD Graphics 640",
        GpuFamily::IntelHDGraphics650 => "Intel HD Graphics 650",
        GpuFamily::IntelHDGraphicsP630 => "Intel HD Graphics P630",
        GpuFamily::IntelIrisPlus640 => "Intel Iris Plus 640",
        GpuFamily::IntelIrisPlus645 => "Intel Iris Plus 645",
        GpuFamily::IntelIrisPlus650 => "Intel Iris Plus 650",
        GpuFamily::IntelIrisPro580 => "Intel Iris Pro 580",
        GpuFamily::IntelIrisXe    => "Intel Iris Xe",
        GpuFamily::IntelIrisXeMax => "Intel Iris Xe MAX",
        GpuFamily::IntelArcA310   => "Intel Arc A310",
        GpuFamily::IntelArcA380   => "Intel Arc A380",
        GpuFamily::IntelArcA580   => "Intel Arc A580",
        GpuFamily::IntelArcA750   => "Intel Arc A750",
        GpuFamily::IntelArcA770   => "Intel Arc A770",
        GpuFamily::IntelArcB580   => "Intel Arc B580",
        GpuFamily::IntelArcB770   => "Intel Arc B770",
        GpuFamily::IntelXeLLVM    => "Intel Xe²",

        // S3
        GpuFamily::S3Trio64       => "S3 Trio64",
        GpuFamily::S3Virge        => "S3 ViRGE",
        GpuFamily::S3Savage3D     => "S3 Savage3D",
        GpuFamily::S3Savage4      => "S3 Savage4",
        GpuFamily::S3Savage2000   => "S3 Savage2000",
        GpuFamily::S3ChromeS20    => "S3 Chrome S20",
        GpuFamily::S3ChromeS25    => "S3 Chrome S25",
        GpuFamily::S3ChromeS27    => "S3 Chrome S27",
        GpuFamily::S3DeltaChrome  => "S3 DeltaChrome",
        GpuFamily::S3GammaChrome  => "S3 GammaChrome",

        // Matrox
        GpuFamily::MatroxMGA      => "Matrox Millennium",
        GpuFamily::MatroxG100     => "Matrox Millennium G100",
        GpuFamily::MatroxG200     => "Matrox Millennium G200",
        GpuFamily::MatroxG400     => "Matrox Millennium G400",
        GpuFamily::MatroxG450     => "Matrox Millennium G450",
        GpuFamily::MatroxG550     => "Matrox Millennium G550",
        GpuFamily::MatroxParhelia => "Matrox Parhelia",
        GpuFamily::MatroxP650     => "Matrox P650",
        GpuFamily::MatroxP690     => "Matrox P690",
        GpuFamily::MatroxM9148    => "Matrox M9148",

        // SiS
        GpuFamily::SiS300         => "SiS 300",
        GpuFamily::SiS315         => "SiS 315",
        GpuFamily::SiS330         => "SiS 330",
        GpuFamily::SiS340         => "SiS 340",
        GpuFamily::SiSXabre200    => "SiS Xabre 200",
        GpuFamily::SiSXabre400    => "SiS Xabre 400",
        GpuFamily::SiSXabre600    => "SiS Xabre 600",
        GpuFamily::SiSMirage      => "SiS Mirage",
        GpuFamily::SiSMirage2     => "SiS Mirage 2",
        GpuFamily::SiSMirage3     => "SiS Mirage 3",
        GpuFamily::SiSM771        => "SiS M771",

        // Rendition
        GpuFamily::RenditionV1000 => "Rendition Vérité V1000",
        GpuFamily::RenditionV2000 => "Rendition Vérité V2000",
        GpuFamily::RenditionV2200 => "Rendition Vérité V2200",

        // PowerVR
        GpuFamily::PowerVRPCX1    => "PowerVR PCX1",
        GpuFamily::PowerVRPCX2    => "PowerVR PCX2",
        GpuFamily::PowerVRKyro    => "PowerVR Kyro",
        GpuFamily::PowerVRKyro2   => "PowerVR Kyro II",
        GpuFamily::PowerVRMBX     => "PowerVR MBX",
        GpuFamily::PowerVR5SGX    => "PowerVR SGX 5 Series",
        GpuFamily::PowerVR6Rogue  => "PowerVR 6 Series (Rogue)",
        GpuFamily::PowerVR7XT     => "PowerVR 7 Series (XT)",

        // Number Nine
        GpuFamily::NumberNineImagine128 => "Number Nine Imagine 128",
        GpuFamily::NumberNineRevolution4 => "Number Nine Revolution 4",

        // Trident
        GpuFamily::TridentProVidia9680 => "Trident ProVidia 9680",
        GpuFamily::Trident3DImage9750 => "Trident 3DImage 9750",
        GpuFamily::Trident3DImage9850 => "Trident 3DImage 9850",
        GpuFamily::TridentCyberBlade => "Trident CyberBlade",
        GpuFamily::TridentCyberBladeXP => "Trident CyberBlade XP",
        GpuFamily::TridentBlade3D  => "Trident Blade 3D",
        GpuFamily::TridentXP4      => "Trident XP4",

        // XGI
        GpuFamily::XGIVolariV3    => "XGI Volari V3",
        GpuFamily::XGIVolariV5    => "XGI Volari V5",
        GpuFamily::XGIVolariV8    => "XGI Volari V8",
        GpuFamily::XGIVolariZ7    => "XGI Volari Z7",
        GpuFamily::XGIVolariZ9    => "XGI Volari Z9",

        // Realtek
        GpuFamily::RealtekRV280   => "Realtek RV280",
        GpuFamily::RealtekRV380   => "Realtek RV380",

        // ARM Mali
        GpuFamily::ARMMali400     => "ARM Mali-400 MP",
        GpuFamily::ARMMali450     => "ARM Mali-450 MP",
        GpuFamily::ARMMaliT604    => "ARM Mali-T604",
        GpuFamily::ARMMaliT628    => "ARM Mali-T628",
        GpuFamily::ARMMaliT760    => "ARM Mali-T760",
        GpuFamily::ARMMaliT860    => "ARM Mali-T860",
        GpuFamily::ARMMaliG71     => "ARM Mali-G71",
        GpuFamily::ARMMaliG76     => "ARM Mali-G76",
        GpuFamily::ARMMaliG78     => "ARM Mali-G78",
        GpuFamily::ARMMaliG310    => "ARM Mali-G310",

        // Qualcomm Adreno
        GpuFamily::Adreno200      => "Qualcomm Adreno 200",
        GpuFamily::Adreno205      => "Qualcomm Adreno 205",
        GpuFamily::Adreno220      => "Qualcomm Adreno 220",
        GpuFamily::Adreno225      => "Qualcomm Adreno 225",
        GpuFamily::Adreno302      => "Qualcomm Adreno 302",
        GpuFamily::Adreno305      => "Qualcomm Adreno 305",
        GpuFamily::Adreno320      => "Qualcomm Adreno 320",
        GpuFamily::Adreno330      => "Qualcomm Adreno 330",
        GpuFamily::Adreno405      => "Qualcomm Adreno 405",
        GpuFamily::Adreno418      => "Qualcomm Adreno 418",
        GpuFamily::Adreno420      => "Qualcomm Adreno 420",
        GpuFamily::Adreno430      => "Qualcomm Adreno 430",
        GpuFamily::Adreno504      => "Qualcomm Adreno 504",
        GpuFamily::Adreno505      => "Qualcomm Adreno 505",
        GpuFamily::Adreno506      => "Qualcomm Adreno 506",
        GpuFamily::Adreno508      => "Qualcomm Adreno 508",
        GpuFamily::Adreno509      => "Qualcomm Adreno 509",
        GpuFamily::Adreno510      => "Qualcomm Adreno 510",
        GpuFamily::Adreno512      => "Qualcomm Adreno 512",
        GpuFamily::Adreno530      => "Qualcomm Adreno 530",
        GpuFamily::Adreno540      => "Qualcomm Adreno 540",
        GpuFamily::Adreno615      => "Qualcomm Adreno 615",
        GpuFamily::Adreno616      => "Qualcomm Adreno 616",
        GpuFamily::Adreno618      => "Qualcomm Adreno 618",
        GpuFamily::Adreno619      => "Qualcomm Adreno 619",
        GpuFamily::Adreno620      => "Qualcomm Adreno 620",
        GpuFamily::Adreno630      => "Qualcomm Adreno 630",
        GpuFamily::Adreno640      => "Qualcomm Adreno 640",
        GpuFamily::Adreno650      => "Qualcomm Adreno 650",
        GpuFamily::Adreno660      => "Qualcomm Adreno 660",
        GpuFamily::Adreno670      => "Qualcomm Adreno 670",
        GpuFamily::Adreno680      => "Qualcomm Adreno 680",
        GpuFamily::Adreno730      => "Qualcomm Adreno 730",
        GpuFamily::Adreno740      => "Qualcomm Adreno 740",
        GpuFamily::Adreno750      => "Qualcomm Adreno 750",

        // Broadcom
        GpuFamily::VideoCoreIV    => "Broadcom VideoCore IV",
        GpuFamily::VideoCoreV     => "Broadcom VideoCore V",
        GpuFamily::VideoCoreVI    => "Broadcom VideoCore VI",

        // Mediatek
        GpuFamily::MediatekMaliG52  => "MediaTek Mali-G52",
        GpuFamily::MediatekMaliG57  => "MediaTek Mali-G57",
        GpuFamily::MediatekMaliG68  => "MediaTek Mali-G68",
        GpuFamily::MediatekMaliG77  => "MediaTek Mali-G77",
        GpuFamily::MediatekMaliG610 => "MediaTek Mali-G610",
        GpuFamily::MediatekMaliG615 => "MediaTek Mali-G615",

        // Rockchip / Allwinner
        GpuFamily::RockchipMali400  => "Rockchip Mali-400",
        GpuFamily::RockchipMaliT760 => "Rockchip Mali-T760",
        GpuFamily::RockchipMaliT860 => "Rockchip Mali-T860",
        GpuFamily::AllwinnerMali400 => "Allwinner Mali-400",

        // AST
        GpuFamily::AST2000       => "ASpeed AST2000",
        GpuFamily::AST2100       => "ASpeed AST2100",
        GpuFamily::AST2300       => "ASpeed AST2300",
        GpuFamily::AST2400       => "ASpeed AST2400",
        GpuFamily::AST2500       => "ASpeed AST2500",
        GpuFamily::AST2600       => "ASpeed AST2600",

        // Cirrus Logic
        GpuFamily::CirrusLogic5430 => "Cirrus Logic CL-GD5430",
        GpuFamily::CirrusLogic5446 => "Cirrus Logic CL-GD5446",
        GpuFamily::CirrusLogic5464 => "Cirrus Logic CL-GD5464",
        GpuFamily::CirrusLogic5465 => "Cirrus Logic CL-GD5465",
        GpuFamily::CirrusLogic67200 => "Cirrus Logic CL-GD67200",
        GpuFamily::CirrusLogic7548 => "Cirrus Logic CL-GD7548",

        // Virtual
        GpuFamily::VMWareSVGA     => "VMware SVGA II",
        GpuFamily::VMWareSVGA3    => "VMware SVGA 3D",
        GpuFamily::VirtIOGPU      => "VirtIO GPU",
        GpuFamily::VirtIOGPU3D    => "VirtIO GPU 3D (VirGL)",
        GpuFamily::BochsVBE       => "Bochs/QEMU VBE",
        GpuFamily::QXL            => "SPICE QXL",
        GpuFamily::VirtualBoxVGA  => "Oracle VirtualBox VGA",
        GpuFamily::HyperVSynthetic => "Microsoft Hyper-V Synthetic GPU",

        // Legacy
        GpuFamily::StandardVGA    => "Standard VGA",
        GpuFamily::VesaVBE        => "VESA BIOS Extensions",
        GpuFamily::Unknown        => "Unknown GPU",
    }
}

// ============================================================================
// GPU CAPABILITIES — Per-family feature flags
// ============================================================================
#[derive(Debug, Clone, Copy)]
pub struct GpuCaps(u32);

impl GpuCaps {
    pub const NONE: GpuCaps = GpuCaps(0);
    pub const HARDWARE_CURSOR: GpuCaps = GpuCaps(1 << 0);
    pub const ACCEL_2D: GpuCaps = GpuCaps(1 << 1);
    pub const ACCEL_3D: GpuCaps = GpuCaps(1 << 2);
    pub const HARDWARE_BLT: GpuCaps = GpuCaps(1 << 3);
    pub const DOUBLE_BUFFER: GpuCaps = GpuCaps(1 << 4);
    pub const HARDWARE_Z: GpuCaps = GpuCaps(1 << 5);
    pub const TEXTURE_UNITS: GpuCaps = GpuCaps(1 << 6);
    pub const TRILINEAR: GpuCaps = GpuCaps(1 << 7);
    pub const ANTI_ALIASING: GpuCaps = GpuCaps(1 << 8);
    pub const SHADER_V2: GpuCaps = GpuCaps(1 << 9);
    pub const VERTEX_PROGRAM: GpuCaps = GpuCaps(1 << 10);
    pub const FRAMEBUFFER: GpuCaps = GpuCaps(1 << 11);
    pub const VBE: GpuCaps = GpuCaps(1 << 12);
    pub const TESSELATION: GpuCaps = GpuCaps(1 << 13);
    pub const COMPUTE: GpuCaps = GpuCaps(1 << 14);
    pub const RAY_TRACING: GpuCaps = GpuCaps(1 << 15);
    pub const DLSS: GpuCaps = GpuCaps(1 << 16);

    pub fn contains(&self, other: GpuCaps) -> bool {
        (self.0 & other.0) == other.0
    }

    pub fn merge(self, other: GpuCaps) -> GpuCaps {
        GpuCaps(self.0 | other.0)
    }
}

pub fn gpu_capabilities(family: GpuFamily) -> GpuCaps {
    match family {
        // 3dfx — basic 2D/3D, double buffer
        GpuFamily::Voodoo1 | GpuFamily::Voodoo2 | GpuFamily::VoodooRush => {
            GpuCaps::ACCEL_2D.merge(GpuCaps::ACCEL_3D).merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::HARDWARE_CURSOR).merge(GpuCaps::TEXTURE_UNITS)
        }
        GpuFamily::VoodooBanshee | GpuFamily::Voodoo3 => {
            GpuCaps::ACCEL_2D.merge(GpuCaps::ACCEL_3D).merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::HARDWARE_CURSOR).merge(GpuCaps::TEXTURE_UNITS)
        }
        GpuFamily::Voodoo4 | GpuFamily::Voodoo5 => {
            GpuCaps::ACCEL_2D.merge(GpuCaps::ACCEL_3D).merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::HARDWARE_CURSOR).merge(GpuCaps::TEXTURE_UNITS)
                .merge(GpuCaps::TRILINEAR)
        }

        // Early Nvidia
        GpuFamily::Nv1 | GpuFamily::Nv2 => GpuCaps::ACCEL_2D.merge(GpuCaps::HARDWARE_CURSOR),
        GpuFamily::Nv3 | GpuFamily::Nv4 | GpuFamily::Riva128 | GpuFamily::Riva128ZX => {
            GpuCaps::ACCEL_2D.merge(GpuCaps::ACCEL_3D).merge(GpuCaps::HARDWARE_CURSOR)
        }

        GpuFamily::Nv5 | GpuFamily::RivaTNT | GpuFamily::RivaTNT2 => {
            GpuCaps::ACCEL_2D.merge(GpuCaps::ACCEL_3D).merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::HARDWARE_CURSOR).merge(GpuCaps::TEXTURE_UNITS)
                .merge(GpuCaps::HARDWARE_BLT)
        }

        // GeForce 256 - GeForce 4
        GpuFamily::GeForce256 | GpuFamily::GeForceDDR => {
            GpuCaps::ACCEL_2D.merge(GpuCaps::ACCEL_3D).merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::HARDWARE_CURSOR).merge(GpuCaps::TEXTURE_UNITS)
                .merge(GpuCaps::HARDWARE_BLT).merge(GpuCaps::HARDWARE_Z)
        }
        GpuFamily::GeForce2 | GpuFamily::GeForce2MX | GpuFamily::GeForce2GTS |
        GpuFamily::GeForce2Ultra | GpuFamily::GeForce2Go => {
            GpuCaps::ACCEL_2D.merge(GpuCaps::ACCEL_3D).merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::HARDWARE_CURSOR).merge(GpuCaps::TEXTURE_UNITS)
                .merge(GpuCaps::HARDWARE_BLT).merge(GpuCaps::HARDWARE_Z)
                .merge(GpuCaps::TRILINEAR)
        }
        GpuFamily::GeForce3 | GpuFamily::GeForce3Ti => {
            GpuCaps::ACCEL_2D.merge(GpuCaps::ACCEL_3D).merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::HARDWARE_CURSOR).merge(GpuCaps::TEXTURE_UNITS)
                .merge(GpuCaps::HARDWARE_BLT).merge(GpuCaps::HARDWARE_Z)
                .merge(GpuCaps::TRILINEAR).merge(GpuCaps::SHADER_V2)
        }
        // Simplified: everything GeForce4+ has full feature set for its era
        _ if family_belongs_to(family, "geforce4+") => {
            GpuCaps::ACCEL_2D.merge(GpuCaps::ACCEL_3D).merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::HARDWARE_CURSOR).merge(GpuCaps::TEXTURE_UNITS)
                .merge(GpuCaps::HARDWARE_BLT).merge(GpuCaps::HARDWARE_Z)
                .merge(GpuCaps::TRILINEAR)
        }

        // S3
        GpuFamily::S3Trio64 => GpuCaps::ACCEL_2D.merge(GpuCaps::HARDWARE_CURSOR),
        GpuFamily::S3Virge => {
            GpuCaps::ACCEL_2D.merge(GpuCaps::ACCEL_3D).merge(GpuCaps::HARDWARE_CURSOR)
        }
        GpuFamily::S3Savage3D | GpuFamily::S3Savage4 => {
            GpuCaps::ACCEL_2D.merge(GpuCaps::ACCEL_3D).merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::HARDWARE_CURSOR).merge(GpuCaps::TEXTURE_UNITS)
        }
        GpuFamily::S3Savage2000 | GpuFamily::S3ChromeS20 | GpuFamily::S3ChromeS25 |
        GpuFamily::S3ChromeS27 | GpuFamily::S3DeltaChrome | GpuFamily::S3GammaChrome => {
            GpuCaps::ACCEL_2D.merge(GpuCaps::ACCEL_3D).merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::HARDWARE_CURSOR).merge(GpuCaps::TEXTURE_UNITS)
                .merge(GpuCaps::HARDWARE_BLT).merge(GpuCaps::HARDWARE_Z)
        }

        // Matrox
        GpuFamily::MatroxMGA | GpuFamily::MatroxG100 => GpuCaps::ACCEL_2D.merge(GpuCaps::HARDWARE_CURSOR),
        GpuFamily::MatroxG200 | GpuFamily::MatroxG400 | GpuFamily::MatroxG450 | GpuFamily::MatroxG550 => {
            GpuCaps::ACCEL_2D.merge(GpuCaps::ACCEL_3D).merge(GpuCaps::DOUBLE_BUFFER)
                .merge(GpuCaps::HARDWARE_CURSOR).merge(GpuCaps::TEXTURE_UNITS)
                .merge(GpuCaps::HARDWARE_BLT)
        }

        // Virtual
        GpuFamily::VMWareSVGA | GpuFamily::VMWareSVGA3 | GpuFamily::VirtIOGPU |
        GpuFamily::VirtIOGPU3D | GpuFamily::BochsVBE => {
            GpuCaps::ACCEL_2D.merge(GpuCaps::DOUBLE_BUFFER).merge(GpuCaps::HARDWARE_CURSOR)
                .merge(GpuCaps::HARDWARE_BLT).merge(GpuCaps::FRAMEBUFFER)
        }

        GpuFamily::QXL | GpuFamily::VirtualBoxVGA | GpuFamily::HyperVSynthetic => {
            GpuCaps::ACCEL_2D.merge(GpuCaps::DOUBLE_BUFFER).merge(GpuCaps::HARDWARE_CURSOR)
                .merge(GpuCaps::HARDWARE_BLT)
        }

        // AST
        GpuFamily::AST2000 | GpuFamily::AST2100 | GpuFamily::AST2300 |
        GpuFamily::AST2400 | GpuFamily::AST2500 | GpuFamily::AST2600 => {
            GpuCaps::ACCEL_2D.merge(GpuCaps::HARDWARE_CURSOR).merge(GpuCaps::FRAMEBUFFER)
        }

        // Remaining: minimal 2D framebuffer
        _ => GpuCaps::FRAMEBUFFER.merge(GpuCaps::HARDWARE_CURSOR),
    }
}

/// Helper to classify GPU families by era for capability matching
fn family_belongs_to(family: GpuFamily, era: &str) -> bool {
    match era {
        "geforce4+" => matches!(family,
            GpuFamily::GeForce4MX | GpuFamily::GeForce4Ti | GpuFamily::GeForce4 |
            GpuFamily::GeForceFX | GpuFamily::GeForceFX5200 | GpuFamily::GeForceFX5600 |
            GpuFamily::GeForceFX5700 | GpuFamily::GeForceFX5800 | GpuFamily::GeForceFX5900 |
            GpuFamily::GeForceFX5950 | GpuFamily::GeForce6 | GpuFamily::GeForce6200 |
            GpuFamily::GeForce6600 | GpuFamily::GeForce6800 | GpuFamily::GeForce7 |
            GpuFamily::GeForce7300 | GpuFamily::GeForce7400 | GpuFamily::GeForce7600 |
            GpuFamily::GeForce7800 | GpuFamily::GeForce7900 | GpuFamily::GeForce7950 |
            GpuFamily::GeForce8 | GpuFamily::GeForce8300 | GpuFamily::GeForce8400 |
            GpuFamily::GeForce8500 | GpuFamily::GeForce8600 | GpuFamily::GeForce8800 |
            GpuFamily::GeForce9 | GpuFamily::GeForce9600 | GpuFamily::GeForce9800 |
            GpuFamily::GeForceGTX200 | GpuFamily::GTX260 | GpuFamily::GTX280 |
            GpuFamily::GTX285 | GpuFamily::GTX295 | GpuFamily::GeForceGTX400 |
            GpuFamily::GTX460 | GpuFamily::GTX465 | GpuFamily::GTX470 | GpuFamily::GTX480 |
            GpuFamily::GeForceGTX500 | GpuFamily::GTX550 | GpuFamily::GTX560 |
            GpuFamily::GTX570 | GpuFamily::GTX580 | GpuFamily::GTX590 |
            GpuFamily::GeForceGTX600 | GpuFamily::GTX650 | GpuFamily::GTX660 |
            GpuFamily::GTX670 | GpuFamily::GTX680 | GpuFamily::GTX690 |
            GpuFamily::GeForceGTX700 | GpuFamily::GTX760 | GpuFamily::GTX770 |
            GpuFamily::GTX780 | GpuFamily::GTX780Ti | GpuFamily::GTXTitan |
            GpuFamily::GeForceGTX900 | GpuFamily::GTX960 | GpuFamily::GTX970 |
            GpuFamily::GTX980 | GpuFamily::GTX980Ti | GpuFamily::GTXTitanX |
            GpuFamily::GeForceGTX10 | GpuFamily::GTX1050 | GpuFamily::GTX1060 |
            GpuFamily::GTX1070 | GpuFamily::GTX1080 | GpuFamily::GTX1080Ti |
            GpuFamily::GTXTitanXP | GpuFamily::GeForceGTX16 | GpuFamily::GTX1650 |
            GpuFamily::GTX1660 | GpuFamily::GTX1660Super | GpuFamily::GTX1660Ti |
            GpuFamily::GeForceRTX20 | GpuFamily::RTX2060 | GpuFamily::RTX2070 |
            GpuFamily::RTX2080 | GpuFamily::RTX2080Ti | GpuFamily::TitanRTX |
            GpuFamily::GeForceRTX30 | GpuFamily::RTX3060 | GpuFamily::RTX3070 |
            GpuFamily::RTX3080 | GpuFamily::RTX3090 | GpuFamily::RTX3090Ti |
            GpuFamily::GeForceRTX40 | GpuFamily::RTX4060 | GpuFamily::RTX4070 |
            GpuFamily::RTX4080 | GpuFamily::RTX4090 | GpuFamily::GeForceRTX50 |
            GpuFamily::RTX5060 | GpuFamily::RTX5070 | GpuFamily::RTX5080 | GpuFamily::RTX5090 |
            GpuFamily::NvidiaQuadro | GpuFamily::NvidiaTesla |
            GpuFamily::GeForce2MX | GpuFamily::GeForce2GTS | GpuFamily::GeForce2Ultra |
            GpuFamily::GeForce2Go | GpuFamily::GeForce3 | GpuFamily::GeForce3Ti
        ),
        _ => false,
    }
}

// ============================================================================
// GPU DEVICE — Physical GPU with PCI info
// ============================================================================
#[derive(Debug, Clone)]
pub struct GpuDevice {
    pub vendor_id: u16,
    pub device_id: u16,
    pub family: GpuFamily,
    pub name: &'static str,
    pub revision: u8,
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
    pub mmio_base: Option<u64>,
    pub mmio_len: Option<u64>,
    pub framebuffer_base: Option<u64>,
    pub framebuffer_len: Option<u64>,
}

impl GpuDevice {
    pub fn new(vendor: u16, device: u16, bus: u8, slot: u8, func: u8) -> Self {
        let family = identify_gpu(vendor, device);
        let name = gpu_name(family);
        Self {
            vendor_id: vendor,
            device_id: device,
            family,
            name,
            revision: 0,
            bus, slot, func,
            mmio_base: None, mmio_len: None,
            framebuffer_base: None, framebuffer_len: None,
        }
    }

    pub fn capabilities(&self) -> GpuCaps {
        gpu_capabilities(self.family)
    }
}

// ============================================================================
// FRAMEBUFFER MODE
// ============================================================================
#[derive(Debug, Clone, Copy)]
pub struct GpuMode {
    pub width: usize,
    pub height: usize,
    pub bpp: u8,
    pub pitch: usize,
    pub framebuffer_addr: u64,
    pub framebuffer_size: usize,
    pub double_buffered: bool,
}

impl GpuMode {
    pub const fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
            bpp: 32,
            pitch: 1280 * 4,
            framebuffer_addr: 0,
            framebuffer_size: 1280 * 720 * 4,
            double_buffered: false,
        }
    }
}

// ============================================================================
// GPU DRIVER TRAIT
// ============================================================================
pub trait GpuDriver: Send {
    fn init(&mut self, mode: &GpuMode) -> Result<(), &'static str>;
    fn current_mode(&self) -> GpuMode;
    fn present(&mut self);
    fn clear(&mut self, color: Color);
    fn put_pixel(&mut self, x: usize, y: usize, color: Color);
    fn fill_rect(&mut self, rect: Rect, color: Color);
    fn blit(&mut self, src_x: usize, src_y: usize, dst_x: usize, dst_y: usize,
            w: usize, h: usize);
    fn capabilities(&self) -> GpuCaps;
    fn family(&self) -> GpuFamily;
    fn raw_framebuffer(&mut self) -> &mut [u8];
}

// ============================================================================
// FALLBACK FRAMEBUFFER DRIVER
// ============================================================================
pub struct FallbackFbDriver {
    fb: *mut u8,
    mode: GpuMode,
    backbuffer: alloc::vec::Vec<u8>,
    caps: GpuCaps,
}

unsafe impl Send for FallbackFbDriver {}

impl FallbackFbDriver {
    pub fn new(fb: *mut u8, width: usize, height: usize) -> Self {
        let size = width * height * 4;
        Self {
            fb,
            mode: GpuMode {
                width, height, bpp: 32,
                pitch: width * 4,
                framebuffer_addr: fb as u64,
                framebuffer_size: size,
                double_buffered: false,
            },
            backbuffer: alloc::vec![0u8; size],
            caps: GpuCaps::FRAMEBUFFER,
        }
    }
}

impl GpuDriver for FallbackFbDriver {
    fn init(&mut self, _mode: &GpuMode) -> Result<(), &'static str> { Ok(()) }
    fn current_mode(&self) -> GpuMode { self.mode }
    fn present(&mut self) {
        unsafe {
            core::ptr::copy_nonoverlapping(self.backbuffer.as_ptr(), self.fb, self.backbuffer.len());
        }
    }
    fn clear(&mut self, color: Color) {
        let bpp = self.mode.bpp as usize / 8;
        for y in 0..self.mode.height {
            for x in 0..self.mode.width {
                let offset = y * self.mode.pitch + x * bpp;
                if offset + 3 < self.backbuffer.len() {
                    self.backbuffer[offset] = color.b;
                    self.backbuffer[offset + 1] = color.g;
                    self.backbuffer[offset + 2] = color.r;
                    if bpp == 4 { self.backbuffer[offset + 3] = 0; }
                }
            }
        }
    }
    fn put_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.mode.width || y >= self.mode.height { return; }
        let bpp = self.mode.bpp as usize / 8;
        let offset = y * self.mode.pitch + x * bpp;
        if offset + 3 < self.backbuffer.len() {
            self.backbuffer[offset] = color.b;
            self.backbuffer[offset + 1] = color.g;
            self.backbuffer[offset + 2] = color.r;
        }
    }
    fn fill_rect(&mut self, rect: Rect, color: Color) {
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                self.put_pixel(x, y, color);
            }
        }
    }
    fn blit(&mut self, _src_x: usize, _src_y: usize, _dst_x: usize, _dst_y: usize,
            _w: usize, _h: usize) {}
    fn capabilities(&self) -> GpuCaps { self.caps }
    fn family(&self) -> GpuFamily { GpuFamily::StandardVGA }
    fn raw_framebuffer(&mut self) -> &mut [u8] { &mut self.backbuffer }
}

// ============================================================================
// GPU MANAGER
// ============================================================================
pub struct GpuManager {
    pub devices: Vec<GpuDevice>,
    pub active_driver_index: Option<usize>,
    pub active_gpu: Option<GpuDevice>,
    pub current_mode: GpuMode,
}

impl Default for GpuManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuManager {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            active_driver_index: None,
            active_gpu: None,
            current_mode: GpuMode::default(),
        }
    }

    pub fn scan_pci(&mut self) {
        crate::serial_println!("GPU: Scanning PCI bus for graphics devices...");
        for bus in 0..=255u8 {
            for slot in 0..=31u8 {
                for func in 0..=7u8 {
                    let vendor = read_pci_config(bus, slot, func, 0) & 0xFFFF;
                    if vendor == 0xFFFF || vendor == 0x0000 {
                        if func == 0 { break; }
                        continue;
                    }
                    // Class code at PCI config offset 0x0B, subclass at 0x0A, revision at 0x08
                    let class_rev = read_pci_config(bus, slot, func, 8);
                    let class_code = ((class_rev >> 24) & 0xFF) as u8;
                    let subclass = ((class_rev >> 16) & 0xFF) as u8;
                    if class_code == 0x03 || (class_code == 0x00 && subclass == 0x01) {
                        let device = (read_pci_config(bus, slot, func, 0) >> 16) as u16;
                        let vendor_id = vendor as u16;
                        let mut gpu = GpuDevice::new(vendor_id, device, bus, slot, func);
                        let bar0 = read_pci_config(bus, slot, func, 4);
                        if bar0 & 0x1 == 0 {
                            let base = bar0 & 0xFFFFFFF0;
                            gpu.mmio_base = Some(base as u64);
                            gpu.framebuffer_base = Some(base as u64);
                        }
                        let bar2 = read_pci_config(bus, slot, func, 6);
                        if bar2 != 0 && bar2 & 0x1 == 0
                            && gpu.framebuffer_base.is_none() {
                                gpu.framebuffer_base = Some((bar2 & 0xFFFFFFF0) as u64);
                            }
                        gpu.revision = (class_rev & 0xFF) as u8;

                        crate::serial_println!(
                            "GPU: [{}:{}:{}] {} ({:#06x}:{:#06x}) rev:{}",
                            bus, slot, func, gpu.name, vendor_id, device, gpu.revision,
                        );
                        self.devices.push(gpu);
                        break;
                    }
                }
            }
        }
        crate::serial_println!("GPU: Found {} display device(s)", self.devices.len());
    }

    pub fn select_and_init(&mut self, desired_width: usize, desired_height: usize) -> Result<(), &'static str> {
        if self.devices.is_empty() {
            crate::serial_println!("GPU: No PCI GPU found, using VESA fallback");
            return Err("no_gpu");
        }
        self.active_gpu = Some(self.devices[0].clone());
        self.active_driver_index = Some(0);

        // Read current VBE mode from hardware (bootloader may have set it already)
        let cur_bpp = crate::gpu::drivers::vbe_read_bpp();
        let cur_width = crate::gpu::drivers::vbe_read_width() as usize;
        let cur_height = crate::gpu::drivers::vbe_read_height() as usize;
        let bpp: u8 = if cur_bpp > 0 && cur_width == desired_width && cur_height == desired_height {
            cur_bpp as u8
        } else {
            24 // fallback
        };
        let bpp_div = (bpp as usize).div_ceil(8);

        self.current_mode = GpuMode {
            width: desired_width,
            height: desired_height,
            bpp,
            pitch: desired_width * bpp_div,
            framebuffer_addr: self.devices[0].framebuffer_base.unwrap_or(0),
            framebuffer_size: desired_width * desired_height * bpp_div,
            double_buffered: false,
        };
        let gpu = &self.devices[0];
        crate::serial_println!("GPU: Selected {} @ {}x{}x{}bpp [Family: {:?}]",
            gpu.name, desired_width, desired_height, bpp, gpu.family);
        Ok(())
    }
}

// ============================================================================
// PCI CONFIG SPACE ACCESS — x86 IO Ports 0xCF8 / 0xCFC
// ============================================================================
fn read_pci_config(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let address: u32 = 0x80000000u32
        | (bus as u32) << 16
        | (slot as u32) << 11
        | (func as u32) << 8
        | (offset as u32 & 0xFC);
    unsafe {
        core::arch::asm!("out dx, eax", in("dx") 0xCF8u16, in("eax") address);
        let value: u32;
        core::arch::asm!("in eax, dx", out("eax") value, in("dx") 0xCFCu16);
        value
    }
}

/// Identify GPU family from vendor/device PCI IDs (public wrapper)
fn identify_gpu(vendor: u16, device: u16) -> GpuFamily {
    // Use the existing match tables
    match vendor {
        0x121A => match device { // 3dfx
            0x0001 => GpuFamily::Voodoo1,
            0x0002 => GpuFamily::Voodoo2,
            0x0003 | 0x0004 => GpuFamily::VoodooBanshee,
            0x0005 => GpuFamily::Voodoo3,
            0x0007 => GpuFamily::Voodoo4,
            0x0009 => GpuFamily::Voodoo5,
            0x000B => GpuFamily::VoodooRush,
            _ => GpuFamily::Unknown,
        },
        0x10DE => { // Nvidia
            // Map known device IDs, fall back to generic Quadro
            match device {
                0x0010 => GpuFamily::Riva128,
                0x0018 => GpuFamily::Riva128ZX,
                0x0020 => GpuFamily::RivaTNT,
                0x0028 | 0x002C => GpuFamily::RivaTNT2,
                0x0100 => GpuFamily::GeForce256,
                0x0101 => GpuFamily::GeForceDDR,
                0x0110 | 0x0151 | 0x0152 => GpuFamily::GeForce2MX,
                0x0150 => GpuFamily::GeForce2GTS,
                0x0200 => GpuFamily::GeForce3,
                0x0201 => GpuFamily::GeForce3Ti,
                0x0250 | 0x0251 => GpuFamily::GeForce4Ti,
                0x0170 | 0x0171 => GpuFamily::GeForce4MX,
                0x0280 | 0x0281 => GpuFamily::GeForce4Ti,
                0x0301 => GpuFamily::GeForceFX5200,
                0x0302 => GpuFamily::GeForceFX5600,
                0x0303 => GpuFamily::GeForceFX5700,
                0x0308 => GpuFamily::GeForceFX5800,
                0x0330 => GpuFamily::GeForceFX5900,
                0x0331 => GpuFamily::GeForceFX5950,
                0x0041 | 0x00F1 => GpuFamily::GeForce6200,
                0x00F2 => GpuFamily::GeForce6600,
                0x0045 => GpuFamily::GeForce6800,
                0x0160 | 0x01D1 | 0x01D8 => GpuFamily::GeForce7600,
                0x0092 | 0x0290 => GpuFamily::GeForce7900,
                0x0191 | 0x0194 => GpuFamily::GeForce8600,
                0x0400 | 0x0600 | 0x0611 | 0x0420 => GpuFamily::GeForce8800,
                0x06E0 | 0x0609 => GpuFamily::GeForce9800,
                0x05E2 | 0x05E3 => GpuFamily::GTX260,
                0x0604 | 0x0603 => GpuFamily::GTX280,
                0x0C80 | 0x0C81 => GpuFamily::GTX460,
                0x0C82 => GpuFamily::GTX470,
                0x0C83 | 0x0C84 => GpuFamily::GTX480,
                0x1200 | 0x1201 => GpuFamily::GTX560,
                0x1081 => GpuFamily::GTX570,
                0x1080 => GpuFamily::GTX580,
                0x1180 => GpuFamily::GTX680,
                0x1004 => GpuFamily::GTX780,
                0x13C0 => GpuFamily::GTX970,
                0x13C1 => GpuFamily::GTX980,
                0x1C01 | 0x1C02 => GpuFamily::GTX1060,
                0x1B00 | 0x1D81 => GpuFamily::GTX1080,
                0x1D82 => GpuFamily::GTX1080Ti,
                0x1E84 | 0x1E04 => GpuFamily::RTX2080,
                0x1E07 => GpuFamily::RTX2080Ti,
                0x2206 => GpuFamily::RTX3080,
                0x2204 => GpuFamily::RTX3090,
                0x2684 => GpuFamily::RTX4090,
                _ => GpuFamily::NvidiaQuadro,
            }
        },
        0x1002 => { // ATI/AMD
            match device {
                0x4742 | 0x4744 => GpuFamily::RagePro,
                0x4752 | 0x4754 => GpuFamily::RageXL,
                0x4C42 | 0x4C46 => GpuFamily::Rage128Pro,
                0x514C => GpuFamily::Radeon8500,
                0x5159 | 0x5157 => GpuFamily::Radeon7500,
                0x4E44 => GpuFamily::Radeon9700,
                0x4E45 => GpuFamily::Radeon9700Pro,
                0x4E46 | 0x4E48 => GpuFamily::Radeon9800,
                0x5B60 | 0x5B62 => GpuFamily::RadeonX300,
                0x5548 => GpuFamily::RadeonX800,
                0x7100 => GpuFamily::RadeonX1800,
                0x7240 => GpuFamily::RadeonX1900,
                0x9440 => GpuFamily::RadeonHD4870,
                0x9442 => GpuFamily::RadeonHD4850,
                0x68B0 => GpuFamily::RadeonHD5770,
                0x689E => GpuFamily::RadeonHD5870,
                0x6718 => GpuFamily::RadeonHD6970,
                0x6810 => GpuFamily::RadeonHD7970,
                0x6811 => GpuFamily::RadeonHD7950,
                0x6640 => GpuFamily::RadeonR9290,
                0x67DF => GpuFamily::RadeonRX480,
                0x67C7 => GpuFamily::RadeonRX580,
                0x731F => GpuFamily::RadeonRX5700,
                0x73BF => GpuFamily::RadeonRX6800,
                0x73C3 => GpuFamily::RadeonRX6900XT,
                0x744C => GpuFamily::RadeonRX7900XT,
                0x744D => GpuFamily::RadeonRX7900XTX,
                _ => GpuFamily::Unknown,
            }
        },
        0x8086 => { // Intel
            match device {
                0x7800 => GpuFamily::Intel740,
                0x7121 => GpuFamily::Intel810,
                0x2582 | 0x2772 => GpuFamily::Intel915,
                0x0152 | 0x0156 => GpuFamily::IntelHDGraphics2500,
                0x0162 | 0x0166 => GpuFamily::IntelHDGraphics4000,
                0x0412 | 0x0416 => GpuFamily::IntelHDGraphics4400,
                0x0D12 | 0x0D16 => GpuFamily::IntelHDGraphics4600,
                0x0A06 | 0x0A0E => GpuFamily::IntelHDGraphics5000,
                0x1616 => GpuFamily::IntelHDGraphics5500,
                0x5916 => GpuFamily::IntelHDGraphics620,
                0x5912 | 0x591B => GpuFamily::IntelHDGraphics630,
                0x5923 => GpuFamily::IntelIrisPlus640,
                0x9B41 => GpuFamily::IntelIrisXe,
                0x56A0 => GpuFamily::IntelArcA750,
                0x56A2 => GpuFamily::IntelArcA770,
                0x56B0 => GpuFamily::IntelArcB580,
                _ => GpuFamily::IntelHDGraphics,
            }
        },
        0x5333 => match device { // S3
            0x8811 | 0x8812 => GpuFamily::S3Trio64,
            0x5631 => GpuFamily::S3Virge,
            0x8A01 => GpuFamily::S3Savage3D,
            0x8A22 | 0x8A23 => GpuFamily::S3Savage4,
            0x9102 | 0x9103 => GpuFamily::S3Savage2000,
            0x9050 => GpuFamily::S3ChromeS20,
            0x3150 => GpuFamily::S3ChromeS25,
            _ => GpuFamily::S3Virge,
        },
        0x102B => match device { // Matrox
            0x0519 => GpuFamily::MatroxMGA,
            0x0520 => GpuFamily::MatroxG100,
            0x0521 | 0x0522 => GpuFamily::MatroxG200,
            0x0525 => GpuFamily::MatroxG400,
            0x0527 => GpuFamily::MatroxG450,
            0x0528 => GpuFamily::MatroxG550,
            _ => GpuFamily::MatroxG200,
        },
        0x1039 => match device { // SiS
            0x6300 => GpuFamily::SiS300,
            0x6330 => GpuFamily::SiS315,
            0x5030 => GpuFamily::SiSXabre200,
            0x5031 => GpuFamily::SiSXabre400,
            _ => GpuFamily::SiSMirage,
        },
        0x10D2 => match device { // Rendition
            0x0001 => GpuFamily::RenditionV1000,
            0x0002 => GpuFamily::RenditionV2000,
            0x0003 => GpuFamily::RenditionV2200,
            _ => GpuFamily::RenditionV2200,
        },
        0x10E3 => match device { // PowerVR
            0x0001 => GpuFamily::PowerVRPCX1,
            0x0003 => GpuFamily::PowerVRPCX2,
            0x0005 => GpuFamily::PowerVRKyro,
            0x0007 => GpuFamily::PowerVRKyro2,
            _ => GpuFamily::PowerVRKyro,
        },
        0x100E => match device { // Number Nine
            0x8880 | 0x8882 => GpuFamily::NumberNineImagine128,
            _ => GpuFamily::NumberNineImagine128,
        },
        0x1023 => match device { // Trident
            0x9680 | 0x9682 => GpuFamily::TridentProVidia9680,
            0x9750 => GpuFamily::Trident3DImage9750,
            0x9850 => GpuFamily::Trident3DImage9850,
            0x2040 => GpuFamily::TridentCyberBlade,
            _ => GpuFamily::TridentCyberBlade,
        },
        0x18CA => match device { // XGI
            0x0022 => GpuFamily::XGIVolariV3,
            0x0024 => GpuFamily::XGIVolariV5,
            0x0026 => GpuFamily::XGIVolariV8,
            _ => GpuFamily::XGIVolariV8,
        },
        0x1013 => match device { // Cirrus Logic
            0x00B8 => GpuFamily::CirrusLogic5430,
            0x00D0 => GpuFamily::CirrusLogic5446,
            0x00D4 => GpuFamily::CirrusLogic5464,
            _ => GpuFamily::StandardVGA,
        },
        0x1A03 => match device { // AST
            0x2000 => GpuFamily::AST2000,
            0x2010 => GpuFamily::AST2100,
            0x2020 => GpuFamily::AST2300,
            0x2030 => GpuFamily::AST2400,
            0x2040 => GpuFamily::AST2500,
            _ => GpuFamily::StandardVGA,
        },
        0x15AD => match device { // VMware
            0x0405 => GpuFamily::VMWareSVGA,
            0x0406 => GpuFamily::VMWareSVGA3,
            _ => GpuFamily::VMWareSVGA,
        },
        0x1AF4 => match device { // VirtIO
            0x1050 => GpuFamily::VirtIOGPU,
            0x1051 => GpuFamily::VirtIOGPU3D,
            _ => GpuFamily::VirtIOGPU,
        },
        0x1234 => GpuFamily::BochsVBE, // QEMU
        0x1B36 => GpuFamily::QXL, // SPICE
        0x80EE => GpuFamily::VirtualBoxVGA, // VirtualBox
        0x1414 => GpuFamily::HyperVSynthetic, // Hyper-V
        _ => GpuFamily::Unknown,
    }
}