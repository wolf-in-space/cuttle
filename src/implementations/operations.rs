use crate::{
    linefy,
    operations::{Operation, OperationInfo, RegisterSdfRenderOpAppExt},
    shader::lines::Lines,
};
use bevy::app::App;

pub fn plugin(app: &mut App) {
    app.register_sdf_render_operation::<Base>()
        .register_sdf_render_operation::<Union>()
        .register_sdf_render_operation::<SmoothUnion>()
        .register_sdf_render_operation::<Subtract>()
        .register_sdf_render_operation::<SmoothSubtract>()
        .register_sdf_render_operation::<Intersect>()
        .register_sdf_render_operation::<SmoothIntersect>();
}

pub struct Base;
impl Operation for Base {
    fn operation_info() -> OperationInfo {
        OperationInfo {
            value: None,
            snippets: Lines::new(),
            operation: "op".to_owned(),
        }
    }
}

pub struct Union;
impl Operation for Union {
    fn operation_info() -> OperationInfo {
        OperationInfo {
            value: None,
            snippets: linefy! {
                fn sdf_union(r1: SdfResult, r2: SdfResult) -> SdfResult {
                    if r1.distance < r2.distance {
                        return r1;
                    } else {
                        return r2;
                    }
                }
            },
            operation: "sdf_union(result, op)".to_owned(),
        }
    }
}

pub struct SmoothUnion;
impl Operation for SmoothUnion {
    fn operation_info() -> OperationInfo {
        OperationInfo {
            value: None,
            snippets: linefy! {
                fn sdf_smooth_union(r1: SdfResult, r2: SdfResult, smoothness: f32) -> SdfResult {
                    let mix = clamp( 0.5 + 0.5 * (r2.distance - r1.distance) / smoothness, 0.0, 1.0);
                    let distance_correction = smoothness * mix * (1.0 - mix);
                    return SdfResult(
                        mix( r2.distance, r1.distance, mix ) - distance_correction,
                        mix( r2.color, r1.color, mix ),
                    );
                }
            },
            operation: "sdf_smooth_union(result, op, 20.0)".to_owned(),
        }
    }
}

pub struct Intersect;
impl Operation for Intersect {
    fn operation_info() -> OperationInfo {
        OperationInfo {
            value: None,
            snippets: linefy! {
                fn sdf_intersect(r1: SdfResult, r2: SdfResult) -> SdfResult {
                    if r1.distance > r2.distance {
                        return r1;
                    } else {
                        return r2;
                    }
                }
            },
            operation: "sdf_intersect(result, op)".to_owned(),
        }
    }
}

pub struct SmoothIntersect;
impl Operation for SmoothIntersect {
    fn operation_info() -> OperationInfo {
        OperationInfo {
            value: None,
            snippets: linefy! {
                fn sdf_smooth_intersect(r1: SdfResult, r2: SdfResult, smoothness: f32) -> SdfResult {
                    let mix = clamp( 0.5 - 0.5 * (r2.distance - r1.distance) / smoothness, 0.0, 1.0);
                    let distance_correction = smoothness * mix * (1.0 - mix);
                    return SdfResult(
                        mix( r2.distance, r1.distance, mix ) + distance_correction,
                        mix( r2.color, r1.color, mix ),
                    );
                }
            },
            operation: "sdf_smooth_intersect(result, op, 20.0)".to_owned(),
        }
    }
}

pub struct Subtract;
impl Operation for Subtract {
    fn operation_info() -> OperationInfo {
        OperationInfo {
            value: None,
            snippets: linefy! {
                fn sdf_subtract(r1: SdfResult, r2: SdfResult) -> SdfResult {
                    if r1.distance > -r2.distance {
                        return r1;
                    } else {
                        return SdfResult(-r2.distance, r1.color);
                    }
                }
            },
            operation: "sdf_subtract(result, op)".to_owned(),
        }
    }
}

pub struct SmoothSubtract;
impl Operation for SmoothSubtract {
    fn operation_info() -> OperationInfo {
        OperationInfo {
            value: None,
            snippets: linefy! {
                fn sdf_smooth_subtract(r1: SdfResult, r2: SdfResult, smoothness: f32) -> SdfResult {
                    let mix = clamp( 0.5 - 0.5 * ( r1.distance + r2.distance ) / smoothness, 0.0, 1.0 );
                    let distance_correction = smoothness * mix * (1.0 - mix);
                    return SdfResult(
                        mix( r1.distance, -r2.distance, mix ) + distance_correction,
                        r1.color,
                    );
                }
            },
            operation: "sdf_smooth_subtract(result, op, 20.0)".to_owned(),
        }
    }
}
