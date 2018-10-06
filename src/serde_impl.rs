use super::{Arena, Entry, Index, Vec, DEFAULT_CAPACITY};
use core::cmp;
use core::fmt;
use core::iter;
use core::marker::PhantomData;
use serde::de::{Deserialize, Deserializer, SeqAccess, Visitor};
use serde::ser::{Serialize, Serializer};

impl Serialize for Index {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Note: do not change the serialization format, or it may break
        // forward and backward compatibility of serialized data!
        (self.index, self.generation).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Index {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (index, generation) = Deserialize::deserialize(deserializer)?;
        Ok(Index { index, generation })
    }
}

impl<T> Serialize for Arena<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Note: do not change the serialization format, or it may break
        // forward and backward compatibility of serialized data!
        serializer.collect_seq(self.items.iter().map(|entry| match entry {
            Entry::Occupied { generation, value } => Some((generation, value)),
            Entry::Free { .. } => None,
        }))
    }
}

impl<'de, T> Deserialize<'de> for Arena<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ArenaVisitor::new())
    }
}

struct ArenaVisitor<T> {
    marker: PhantomData<fn() -> Arena<T>>,
}

impl<T> ArenaVisitor<T> {
    fn new() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<'de, T> Visitor<'de> for ArenaVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = Arena<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a generational arena")
    }

    fn visit_seq<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: SeqAccess<'de>,
    {
        let init_cap = access.size_hint().unwrap_or(DEFAULT_CAPACITY);
        let mut items = Vec::with_capacity(init_cap);

        let mut generation = 0;
        while let Some(element) = access.next_element::<Option<(u64, T)>>()? {
            let item = match element {
                Some((gen, value)) => {
                    generation = cmp::max(generation, gen);
                    Entry::Occupied {
                        generation: gen,
                        value,
                    }
                }
                None => Entry::Free { next_free: None },
            };
            items.push(item);
        }

        // items.len() must be same as item.capacity(), so fill the unused elements with Free.
        if items.len() + 1 < items.capacity() {
            let add_cap = items.capacity() - (items.len() + 1);
            items.reserve_exact(add_cap);
            items.extend(iter::repeat_with(|| Entry::Free { next_free: None }).take(add_cap));
            debug_assert_eq!(items.len(), items.capacity());
        }

        let mut free_list_head = None;
        let mut len = items.len();
        // Iterates `arena.items` in reverse order so that free_list concatenates
        // indices in ascending order.
        for (idx, entry) in items.iter_mut().enumerate().rev() {
            if let Entry::Free { next_free } = entry {
                *next_free = free_list_head;
                free_list_head = Some(idx);
                len -= 1;
            }
        }

        Ok(Arena {
            items,
            generation,
            free_list_head,
            len,
        })
    }
}
