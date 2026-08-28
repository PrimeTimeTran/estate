use egui::Color32;

// pub const BG: Color32 = Color32::from_rgb(18, 20, 24);
// pub const BORDER: Color32 = Color32::from_rgb(52, 57, 68);
// pub const DANGER: Color32 = Color32::from_rgb(230, 90, 95);
// pub const GRID: Color32 = Color32::from_rgb(45, 49, 58);
// pub const PRIMARY: Color32 = Color32::from_rgb(100, 160, 255);
// pub const SUCCESS: Color32 = Color32::from_rgb(82, 190, 125);
// pub const SURFACE: Color32 = Color32::from_rgb(27, 30, 36);
// pub const SURFACE_HOVER: Color32 = Color32::from_rgb(34, 38, 46);
// pub const TEXT: Color32 = Color32::from_rgb(232, 235, 240);
// pub const TEXT_MUTED: Color32 = Color32::from_rgb(145, 152, 165);
// pub const WARNING: Color32 = Color32::from_rgb(235, 180, 70);

// -----------------------------------------------------------------------------
// MATERIAL-STYLE COLOR TOKENS
// -----------------------------------------------------------------------------
//
// Tokens describe the ROLE of a color rather than its visual appearance.
//
// Primary
//     Main interactive / branded color.
//
// Secondary
//     Supporting accent.
//
// Tertiary
//     Additional accent / differentiation.
//
// Surface
//     Background and container hierarchy.
//
// On-*
//     Content rendered on top of the corresponding color.
//
// Outline
//     Borders and dividers.
//
// Error / Warning / Success
//     Semantic status colors.
// -----------------------------------------------------------------------------

// -----------------------------------------------------------------------------
// PRIMARY
// -----------------------------------------------------------------------------

pub const PRIMARY: Color32 = Color32::from_rgb(208, 188, 255);
pub const ON_PRIMARY: Color32 = Color32::from_rgb(55, 30, 100);
pub const PRIMARY_CONTAINER: Color32 = Color32::from_rgb(79, 55, 139);
pub const ON_PRIMARY_CONTAINER: Color32 = Color32::from_rgb(234, 221, 255);

// -----------------------------------------------------------------------------
// SECONDARY
// -----------------------------------------------------------------------------

pub const SECONDARY: Color32 = Color32::from_rgb(204, 194, 214);
pub const ON_SECONDARY: Color32 = Color32::from_rgb(51, 45, 59);
pub const SECONDARY_CONTAINER: Color32 = Color32::from_rgb(74, 68, 81);
pub const ON_SECONDARY_CONTAINER: Color32 = Color32::from_rgb(232, 222, 238);

// -----------------------------------------------------------------------------
// TERTIARY
// -----------------------------------------------------------------------------

pub const TERTIARY: Color32 = Color32::from_rgb(239, 184, 200);
pub const ON_TERTIARY: Color32 = Color32::from_rgb(73, 37, 49);
pub const TERTIARY_CONTAINER: Color32 = Color32::from_rgb(99, 59, 72);
pub const ON_TERTIARY_CONTAINER: Color32 = Color32::from_rgb(255, 217, 226);

// -----------------------------------------------------------------------------
// SURFACE
// -----------------------------------------------------------------------------

pub const SURFACE: Color32 = Color32::from_rgb(20, 19, 24);
pub const ON_SURFACE: Color32 = Color32::from_rgb(231, 225, 229);

pub const SURFACE_VARIANT: Color32 = Color32::from_rgb(72, 69, 78);
pub const ON_SURFACE_VARIANT: Color32 = Color32::from_rgb(202, 196, 208);

// Material 3 surface-container hierarchy.
//
// Use these instead of inventing arbitrary BG / CARD / PANEL colors.
//
//     SURFACE
//       ↓
//     SURFACE_CONTAINER_LOW
//       ↓
//     SURFACE_CONTAINER
//       ↓
//     SURFACE_CONTAINER_HIGH
//       ↓
//     SURFACE_CONTAINER_HIGHEST

pub const SURFACE_CONTAINER_LOWEST: Color32 = Color32::from_rgb(15, 14, 18);
pub const SURFACE_CONTAINER_LOW: Color32 = Color32::from_rgb(28, 27, 32);
pub const SURFACE_CONTAINER: Color32 = Color32::from_rgb(32, 31, 36);
pub const SURFACE_CONTAINER_HIGH: Color32 = Color32::from_rgb(43, 41, 47);
pub const SURFACE_CONTAINER_HIGHEST: Color32 = Color32::from_rgb(54, 52, 59);

// -----------------------------------------------------------------------------
// SURFACE TINT / INTERACTION
// -----------------------------------------------------------------------------

pub const SURFACE_TINT: Color32 = PRIMARY;

// Useful application-level semantic interaction tokens.
//
// Material doesn't prescribe these exact names as core color roles,
// but they're useful when building an actual UI.

pub const SURFACE_HOVER: Color32 = Color32::from_rgb(45, 43, 50);
pub const SURFACE_PRESSED: Color32 = Color32::from_rgb(58, 55, 63);
pub const SURFACE_SELECTED: Color32 = Color32::from_rgb(60, 52, 78);
pub const SURFACE_DISABLED: Color32 = Color32::from_rgb(30, 29, 34);

// -----------------------------------------------------------------------------
// OUTLINE
// -----------------------------------------------------------------------------

pub const OUTLINE: Color32 = Color32::from_rgb(147, 143, 153);
pub const OUTLINE_VARIANT: Color32 = Color32::from_rgb(73, 69, 79);

// Application-specific divider token.
pub const DIVIDER: Color32 = OUTLINE_VARIANT;

// -----------------------------------------------------------------------------
// TEXT
// -----------------------------------------------------------------------------

// Prefer semantic On-* tokens for text where possible.
//
// These aliases make application code especially readable.

pub const TEXT: Color32 = ON_SURFACE;
pub const TEXT_MUTED: Color32 = ON_SURFACE_VARIANT;

// Additional hierarchy for dense application UIs.
pub const TEXT_SUBTLE: Color32 = Color32::from_rgb(160, 155, 165);
pub const TEXT_DISABLED: Color32 = Color32::from_rgb(120, 117, 124);

// -----------------------------------------------------------------------------
// ERROR
// -----------------------------------------------------------------------------

pub const ERROR: Color32 = Color32::from_rgb(255, 180, 171);
pub const ON_ERROR: Color32 = Color32::from_rgb(105, 0, 5);
pub const ERROR_CONTAINER: Color32 = Color32::from_rgb(147, 0, 10);
pub const ON_ERROR_CONTAINER: Color32 = Color32::from_rgb(255, 218, 214);

// -----------------------------------------------------------------------------
// WARNING
// -----------------------------------------------------------------------------
//
// Warning isn't a core Material 3 color role in the same way that
// primary / secondary / tertiary / error are, but it is useful as
// an application semantic token.

pub const WARNING: Color32 = Color32::from_rgb(255, 186, 73);
pub const ON_WARNING: Color32 = Color32::from_rgb(67, 43, 0);
pub const WARNING_CONTAINER: Color32 = Color32::from_rgb(102, 68, 0);
pub const ON_WARNING_CONTAINER: Color32 = Color32::from_rgb(255, 222, 164);

// -----------------------------------------------------------------------------
// SUCCESS
// -----------------------------------------------------------------------------

pub const SUCCESS: Color32 = Color32::from_rgb(129, 210, 140);
pub const ON_SUCCESS: Color32 = Color32::from_rgb(0, 56, 21);
pub const SUCCESS_CONTAINER: Color32 = Color32::from_rgb(0, 82, 31);
pub const ON_SUCCESS_CONTAINER: Color32 = Color32::from_rgb(158, 241, 168);

// -----------------------------------------------------------------------------
// INFO
// -----------------------------------------------------------------------------

pub const INFO: Color32 = Color32::from_rgb(164, 200, 255);
pub const ON_INFO: Color32 = Color32::from_rgb(0, 48, 94);
pub const INFO_CONTAINER: Color32 = Color32::from_rgb(0, 72, 135);
pub const ON_INFO_CONTAINER: Color32 = Color32::from_rgb(207, 226, 255);

// -----------------------------------------------------------------------------
// DATA VISUALIZATION
// -----------------------------------------------------------------------------
//
// These are deliberately semantic rather than named RED / BLUE / GREEN.
// This gives charts a stable vocabulary without coupling application
// code to particular hues.

pub const CHART_PRIMARY: Color32 = PRIMARY;
pub const CHART_SECONDARY: Color32 = SECONDARY;
pub const CHART_TERTIARY: Color32 = TERTIARY;
pub const CHART_POSITIVE: Color32 = SUCCESS;
pub const CHART_NEGATIVE: Color32 = ERROR;
pub const CHART_NEUTRAL: Color32 = TEXT_MUTED;
pub const CHART_GRID: Color32 = OUTLINE_VARIANT;

// -----------------------------------------------------------------------------
// LEGACY / CONVENIENCE ALIASES
// -----------------------------------------------------------------------------
//
// Keep these only if they make existing application code clearer.

pub const BG: Color32 = SURFACE;
pub const BORDER: Color32 = OUTLINE_VARIANT;
pub const GRID: Color32 = CHART_GRID;
