pub use crate::gamepad::{
    GamepadBinding, GamepadBindingSource, GamepadBindings, GamepadHapticOutput,
    GamepadHapticOutputBindings, GamepadPathSelector,
};
pub use crate::keyboard::{KeyboardBinding, KeyboardBindings};
pub use crate::mouse::{MouseBindings, MouseButtonBinding, MouseMotionBinding, MouseMotionType};
// these all work with only "xr" by chance, nice
#[cfg(feature = "xr")]
pub use crate::openxr::{
    OxrBindings, META_TOUCH_PLUS_PROFILE, META_TOUCH_PRO_PROFILE, OCULUS_TOUCH_PROFILE,
};
pub use crate::subaction_paths::{RequestedSubactionPaths, SubactionPaths};
#[cfg(feature = "xr")]
pub use crate::xr::{AttachSpaceToEntity, SpaceActionValue};
pub use crate::DefaultSchminputPlugins;
pub use crate::{Action, ActionSet};
pub use crate::{BoolActionValue, F32ActionValue, Vec2ActionValue};
