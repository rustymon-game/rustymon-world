use pest::iterators::{Pair, Pairs};
use pest::RuleType;
use std::mem::MaybeUninit;

pub trait PairsExt<'i, T> {
    fn child(self) -> Option<Pair<'i, T>>;
    fn children<const N: usize>(self) -> Option<[Pair<'i, T>; N]>;
    fn head_tail(self) -> Option<(Pair<'i, T>, Pairs<'i, T>)>;
}
impl<'i, T> PairsExt<'i, T> for Pair<'i, T>
where
    T: RuleType,
{
    fn child(self) -> Option<Pair<'i, T>> {
        self.into_inner().next()
    }
    fn children<const N: usize>(self) -> Option<[Pair<'i, T>; N]> {
        let mut array: [MaybeUninit<Pair<'i, T>>; N] =
            unsafe { MaybeUninit::uninit().assume_init() };

        let mut children = self.into_inner();
        for slot in &mut array {
            slot.write(children.next()?);
        }

        Some(array.map(|pair| unsafe { MaybeUninit::assume_init(pair) }))
    }
    fn head_tail(self) -> Option<(Pair<'i, T>, Pairs<'i, T>)> {
        let mut pairs = self.into_inner();
        let head = pairs.next()?;
        Some((head, pairs))
    }
}
