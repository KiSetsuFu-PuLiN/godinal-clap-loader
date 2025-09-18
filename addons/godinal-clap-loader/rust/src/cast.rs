use godot::prelude::*;
use std::any::type_name;

pub fn assert_cast<T: GodotClass, D: Inherits<T>>(source: Gd<T>) -> Result<Gd<D>, Gd<T>> {
    let result = source.try_cast();
    if result.is_err() {
        godot_error!(
            "进行从{}到{}的类型转换时失败。",
            type_name::<T>(),
            type_name::<D>()
        );
    }
    result
}
