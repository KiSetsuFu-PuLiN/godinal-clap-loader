use godot::{classes::WeakRef, global::weakref, prelude::*};

pub fn assert_to_weak_ref<T: Inherits<RefCounted>>(ref_counted: &Gd<T>) -> Option<Gd<WeakRef>> {
    match weakref(&ref_counted.to_variant()).try_to() {
        Ok(weak_ref) => Some(weak_ref),
        Err(convert_error) => {
            godot_error!("所给对象无法转换为弱引用：{convert_error}");
            None
        }
    }
}

pub fn assert_from_weak_ref<T: Inherits<RefCounted>>(weak_ref: &Gd<WeakRef>) -> Option<Gd<T>> {
    let variant = weak_ref.get_ref();
    if variant.is_nil() {
        return None;
    }
    match variant.try_to() {
        Ok(ref_counted) => Some(ref_counted),
        Err(convert_error) => {
            godot_error!("弱引用无法转换为指定类型：{convert_error}");
            None
        }
    }
}
