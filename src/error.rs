use std::error::Error;
use std::fmt;

/// 几何计算错误。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmoothError {
    /// 输入包含 `NaN` 或不允许的无穷值。
    NonFiniteInput,
    /// 长度、半径或限制为负数。
    NegativeInput,
    /// 角点方向向量长度过小。
    DegenerateAxis,
    /// 角点夹角不在开区间 `(0, PI)` 内。
    InvalidAngle,
    /// 闭合 frame 少于 3 个点。
    TooFewPoints,
    /// frame 包含退化边、共线点或面积为零。
    DegenerateFrame,
    /// frame 存在凹角，当前版本暂不支持。
    ConcaveFrame,
    /// frame 存在自相交边。
    SelfIntersectingFrame,
}

impl fmt::Display for SmoothError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SmoothError::NonFiniteInput => write!(f, "输入包含非有限数值"),
            SmoothError::NegativeInput => write!(f, "长度、半径或限制不能为负数"),
            SmoothError::DegenerateAxis => write!(f, "角点方向向量退化"),
            SmoothError::InvalidAngle => write!(f, "角点角度必须满足 0 < phi < PI"),
            SmoothError::TooFewPoints => write!(f, "闭合 frame 至少需要 3 个点"),
            SmoothError::DegenerateFrame => write!(f, "frame 包含退化边或面积为零"),
            SmoothError::ConcaveFrame => write!(f, "当前版本仅支持凸 frame"),
            SmoothError::SelfIntersectingFrame => write!(f, "当前版本不支持自相交 frame"),
        }
    }
}

impl Error for SmoothError {}
