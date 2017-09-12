use super::Cache;

pub(crate) trait CacheUpdate {
    type Output;

    fn update(&mut self, &mut Cache) -> Option<Self::Output>;
}
