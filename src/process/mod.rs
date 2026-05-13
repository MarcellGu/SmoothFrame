/// 单角 smooth corner 的纯几何处理。
pub mod corner;

/// 处理层内部统一 trait：输入层传入已校验的数据，处理层只负责几何计算。
///
/// 这个 trait 和它的具体 processor 实现都保持 crate 私有，避免外部依赖内部几何流水线；
/// 公开扩展点放在 output 层，由 [`crate::PathFormatter`] 承担。
pub(crate) trait Processor {
    /// 处理后的几何结果。
    type Output;

    /// 执行纯几何处理。
    fn process(&self) -> Self::Output;
}
