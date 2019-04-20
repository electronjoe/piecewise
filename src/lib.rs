use intervals_general::interval::Interval;
use itertools::iproduct;
use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;
use smallvec::smallvec;
use std::iter::once;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ValueOverInterval<T, U> {
    pub(crate) interval: Interval<T>,
    pub(crate) value: U,
}

impl<T, U> ValueOverInterval<T, U> {
    /// Create a new ValueOverInterval
    ///
    /// # Examples
    ///
    /// ```
    /// use intervals_general::interval::Interval;
    /// use piecewise::ValueOverInterval;
    ///
    /// let value_over_interval = ValueOverInterval::new(Interval::Unbounded::<u32> {}, 5.0);
    /// ```
    pub fn new(interval: Interval<T>, value: U) -> ValueOverInterval<T, U> {
        ValueOverInterval { interval, value }
    }

    /// Fetch an immutable reference to the Interval
    ///
    /// # Examples
    ///
    /// ```
    /// use intervals_general::bound_pair::BoundPair;
    /// use intervals_general::interval::Interval;
    /// use piecewise::ValueOverInterval;
    /// # fn main() -> std::result::Result<(), String> {
    /// let bounds = BoundPair::new(1.0, 2.0).ok_or("invalid BoundPair")?;
    /// let value_over_interval = ValueOverInterval::new(Interval::Closed { bound_pair: bounds }, 4);
    /// assert_eq!(*value_over_interval.value(), 4);
    /// # Ok(())
    /// # }
    /// ```
    pub fn interval(&self) -> &Interval<T> {
        &self.interval
    }

    /// Fetch an immutable reference to the value
    ///
    /// # Examples
    ///
    /// ```
    /// use intervals_general::bound_pair::BoundPair;
    /// use intervals_general::interval::Interval;
    /// use piecewise::ValueOverInterval;
    /// # fn main() -> std::result::Result<(), String> {
    /// let bounds = BoundPair::new(1.0, 2.0).ok_or("invalid BoundPair")?;
    /// let value_over_interval = ValueOverInterval::new(Interval::Closed { bound_pair: bounds }, 4);
    /// assert_eq!(
    ///     *value_over_interval.interval(),
    ///     Interval::Closed { bound_pair: bounds }
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn value(&self) -> &U {
        &self.value
    }
}

type ValueOverIntervalOptionalTuple<T, U, V> = (
    Option<ValueOverInterval<T, U>>,
    Option<ValueOverInterval<T, V>>,
);

impl<T, U, V> std::ops::Mul<V> for ValueOverInterval<T, U>
where
    U: std::ops::Mul<V, Output = U>,
{
    type Output = ValueOverInterval<T, U>;

    /// Implementation of Mul for ValueOverInterval
    ///
    /// # Example
    ///
    /// ```
    /// use intervals_general::bound_pair::BoundPair;
    /// use intervals_general::interval::Interval;
    /// use piecewise::ValueOverInterval;
    /// # fn main() -> std::result::Result<(), String> {
    /// let bounds = BoundPair::new(1.0, 2.0).ok_or("invalid BoundPair")?;
    /// let value_over_interval =
    ///     ValueOverInterval::new(Interval::Closed { bound_pair: bounds }, 4);
    /// let scaled_value = value_over_interval * 12;
    /// assert_eq!(*scaled_value.value(), 48);
    /// # Ok(())
    /// # }
    fn mul(self, rhs: V) -> Self::Output {
        ValueOverInterval {
            interval: self.interval,
            value: self.value * rhs,
        }
    }
}

/// SmallPiecewise
///
/// The SmallPiecewise variant is for use when the number of Intervals
/// over which the function is defined is relatively small.  For these
/// small entities, benchmarking backs the intuition that we benefit from:
///
/// * Stack storage with heap overflow (via SmallVec)
/// * Linear search instead of binary search
///
/// The step function is ensured to be well-defined. Specifically this means:
///
/// * All intervals are pairwise disjoint (non-overlapping)
/// * The union of the intervals is the entire real line (TODO: drop this?)
///
/// These guarantees are ensured by the StepFunctionBuilder at build() time, and
/// by operations over piecewise functions.
#[derive(Clone, Debug)]
pub struct SmallPiecewise<T, U> {
    values_over_intervals: smallvec::SmallVec<[ValueOverInterval<T, U>; 8]>,
}

impl<T, U> SmallPiecewise<T, U>
where
    T: std::cmp::PartialOrd,
    T: std::marker::Copy,
{
    /// Private creation exclusively for crate operations
    ///
    /// For public construction use SmallPiecewiseBuilder
    fn new() -> SmallPiecewise<T, U> {
        SmallPiecewise {
            values_over_intervals: smallvec::SmallVec::new(),
        }
    }

    /// Private add for crate use in known well defined SmallPiecewise segments
    ///
    /// Must add segments in sorted order by right bound (nominal storage order),
    /// and segmenets must not overlap. These guarantees are made by the crate
    /// in all use of this private function.
    fn add(&mut self, element: ValueOverInterval<T, U>) -> &mut Self {
        self.values_over_intervals.push(element);
        self
    }

    /// Retrieves the value of the piecewise function at a specific point
    ///
    /// If the Domain does not contain the value specified by at: - Optional
    /// returns None
    ///
    /// # Runtime
    ///
    /// Using a simple learning search - runtime is linear in Segment count
    ///
    /// # Examples
    ///
    /// ```
    /// use intervals_general::interval::Interval;
    /// use piecewise::SmallPiecewiseBuilder;
    /// use piecewise::ValueOverInterval;
    ///
    /// let mut builder: SmallPiecewiseBuilder<u32, f32> = SmallPiecewiseBuilder::new();
    /// builder.add(ValueOverInterval::new(
    ///     Interval::UnboundedClosedLeft { left: 230 },
    ///     2.0,
    /// ));
    /// builder.add(ValueOverInterval::new(
    ///     Interval::UnboundedOpenRight { right: 200 },
    ///     1.0,
    /// ));
    /// let small_piecewise = builder.build();
    ///
    /// assert_eq!(small_piecewise.value_at(1), Some(&1.0));
    /// assert_eq!(small_piecewise.value_at(200), None);
    /// assert_eq!(small_piecewise.value_at(230), Some(&2.0));
    /// ```
    pub fn value_at(&self, at: T) -> Option<&U> {
        self.values_over_intervals
            .iter()
            .find(|voi| voi.interval().contains(&Interval::Singleton { at }))
            .and_then(|voi| Some(voi.value()))
    }
}

impl<T, U> std::fmt::Display for SmallPiecewise<T, U>
where
    T: std::fmt::Debug,
    U: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut output = String::new();
        for i in self.values_over_intervals.iter() {
            output.push_str(&format!("{}{: >7?}\n", i.interval(), i.value()));
        }
        write!(f, "{}", output)
    }
}

impl<T, U> std::iter::FromIterator<ValueOverInterval<T, U>> for SmallPiecewise<T, U>
where
    T: std::cmp::PartialOrd,
    T: std::marker::Copy,
{
    fn from_iter<I: IntoIterator<Item = ValueOverInterval<T, U>>>(iter: I) -> Self {
        let mut c = SmallPiecewise::new();

        for i in iter {
            c.add(i);
        }

        c
    }
}

/// Multiply two SmallPiecewise point-wise across Domain
///
/// For every point defined in both Piecewise functions, multiply the values
/// and form an output interval accordingly.  For regions of the domain having
/// only one of two SmallPiecewise defined, the output is undefined.
///
/// # Examples
///
/// ```
/// use intervals_general::interval::Interval;
/// use piecewise::SmallPiecewiseBuilder;
/// use piecewise::ValueOverInterval;
///
/// let mut builder: SmallPiecewiseBuilder<u32, f32> = SmallPiecewiseBuilder::new();
/// builder.add(ValueOverInterval::new(
///     Interval::UnboundedClosedLeft { left: 230 },
///     2.0,
/// ));
/// builder.add(ValueOverInterval::new(
///     Interval::UnboundedOpenRight { right: 200 },
///     1.0,
/// ));
/// let piecewise_1 = builder.build();
///
/// builder = SmallPiecewiseBuilder::new();
/// builder.add(ValueOverInterval::new(
///     Interval::UnboundedClosedLeft { left: 180 },
///     -10.0,
/// ));
/// let piecewise_2 = builder.build();
///
/// let result = piecewise_1 * piecewise_2;
///
/// assert_eq!(result.value_at(1), None);
/// assert_eq!(result.value_at(190), Some(&-10.0));
/// assert_eq!(result.value_at(200), None);
/// assert_eq!(result.value_at(230), Some(&-20.0));
/// ```
impl<T, U, V> std::ops::Mul<SmallPiecewise<T, V>> for SmallPiecewise<T, U>
where
    T: Copy,
    T: PartialOrd,
    U: Copy,
    U: std::ops::Mul<V>,
    V: Copy,
    <U as std::ops::Mul<V>>::Output: Copy + Clone,
    SmallPiecewise<T, <U as std::ops::Mul<V>>::Output>:
        std::iter::FromIterator<ValueOverInterval<T, U>>,
{
    type Output = SmallPiecewise<T, <U as std::ops::Mul<V>>::Output>;

    fn mul(self, rhs: SmallPiecewise<T, V>) -> Self::Output {
        let mut prior_intervals: ValueOverIntervalOptionalTuple<T, U, V> = (None, None);

        self.values_over_intervals
            .iter()
            .merge_join_by(rhs.values_over_intervals.iter(), |a, b| {
                if let Some(cmp) = a.interval.right_partial_cmp(&b.interval) {
                    cmp
                } else {
                    // TODO(smoeller) fix this... how? This occures due to e.g. Empty interval
                    std::cmp::Ordering::Less
                }
            })
            .flat_map(|either| {
                // TODO(smoeller) since using once / chain - no longer need inner Options?
                // Alternative is to implement a None, One or Two enum that is iterable
                match either {
                    Left(new_left) => {
                        let retval = if let (.., Some(ref right)) = prior_intervals {
                            once(Some(ValueOverInterval::new(
                                new_left.interval.intersect(&right.interval),
                                new_left.value * right.value,
                            )))
                            .chain(once(None))
                        } else {
                            once(None).chain(once(None))
                        };
                        prior_intervals = (Some(*new_left), prior_intervals.1);
                        retval
                    }
                    Right(new_right) => {
                        let retval = if let (Some(ref left), ..) = prior_intervals {
                            once(Some(ValueOverInterval::new(
                                left.interval.intersect(&new_right.interval),
                                left.value * new_right.value,
                            )))
                            .chain(once(None))
                        } else {
                            once(None).chain(once(None))
                        };
                        prior_intervals = (prior_intervals.0, Some(*new_right));
                        retval
                    }
                    Both(new_left, new_right) => {
                        let new_right_induced = if let (Some(ref left), ..) = prior_intervals {
                            Some(ValueOverInterval::new(
                                left.interval.intersect(&new_right.interval),
                                left.value * new_right.value,
                            ))
                        } else {
                            None
                        };
                        let new_left_induced = if let (.., Some(ref right)) = prior_intervals {
                            Some(ValueOverInterval::new(
                                new_left.interval.intersect(&right.interval),
                                new_left.value * right.value,
                            ))
                        } else {
                            None
                        };
                        // Take new_right_induced if Some and not Interval::Empty, else new_left_induced
                        let first = match new_right_induced {
                            None
                            | Some(ValueOverInterval {
                                interval: Interval::Empty,
                                ..
                            }) => once(new_left_induced),
                            _ => once(new_right_induced),
                        };
                        // Tag on any interval generated by the combined new intervals
                        let retval = first.chain(once(Some(ValueOverInterval::new(
                            new_left.interval.intersect(&new_right.interval),
                            new_left.value * new_right.value,
                        ))));
                        prior_intervals = (Some(*new_left), Some(*new_right));
                        retval
                    }
                }
            })
            .filter_map(|x| x)
            .collect()
    }
}

#[derive(Default)]
pub struct SmallPiecewiseBuilder<T, U>
where
    T: Copy,
    T: PartialOrd,
{
    values_over_intervals: smallvec::SmallVec<[ValueOverInterval<T, U>; 8]>,
}

impl<T, U> SmallPiecewiseBuilder<T, U>
where
    T: std::cmp::PartialOrd,
    T: std::marker::Copy,
    U: std::marker::Copy,
{
    pub fn new() -> SmallPiecewiseBuilder<T, U> {
        SmallPiecewiseBuilder {
            values_over_intervals: smallvec::SmallVec::new(),
        }
    }

    /// Consume the builder and produce a SmallPiecewise output
    pub fn build(self) -> SmallPiecewise<T, U> {
        SmallPiecewise {
            values_over_intervals: self.values_over_intervals,
        }
    }

    /// Add a Segment to the Builder, overlay on top of existing
    ///
    /// When adding a new Segment, if portions of the existing Segments
    /// overlap in the domain, the new segment is applied and existing
    /// segments are modified to deconflict (newest addition wins).
    ///
    /// Additionally, the segments are sorted and duplicates are removed.
    ///
    /// # Example
    ///
    /// ```
    /// use intervals_general::interval::Interval;
    /// use piecewise::SmallPiecewiseBuilder;
    /// use piecewise::ValueOverInterval;
    ///
    /// let mut builder: SmallPiecewiseBuilder<u32, f32> = SmallPiecewiseBuilder::new();
    /// builder.add(ValueOverInterval::new(Interval::Unbounded, 5.0));
    /// builder.add(ValueOverInterval::new(
    ///     Interval::UnboundedClosedLeft { left: 230 },
    ///     2.0,
    /// ));
    /// builder.add(ValueOverInterval::new(
    ///     Interval::UnboundedOpenRight { right: 200 },
    ///     1.0,
    /// ));
    /// let small_piecewise = builder.build();
    ///
    /// println!("{}", small_piecewise);
    ///
    /// assert_eq!(small_piecewise.value_at(1), Some(&1.0));
    /// assert_eq!(small_piecewise.value_at(210), Some(&5.0));
    /// assert_eq!(small_piecewise.value_at(230), Some(&2.0));
    /// assert_eq!(small_piecewise.value_at(231), Some(&2.0));
    /// ```
    pub fn add(&mut self, element: ValueOverInterval<T, U>) -> &mut Self {
        let mut new_voi: smallvec::SmallVec<[ValueOverInterval<T, U>; 8]> = smallvec![];
        for (self_voi, complement_interval) in
            iproduct!(&self.values_over_intervals, element.interval().complement())
        {
            let intersection = self_voi.interval().intersect(&complement_interval);
            if let Interval::Empty = intersection {
                // Empty interval ValueOverInterval are not meaningful, discard
            } else {
                new_voi.push(ValueOverInterval {
                    interval: intersection,
                    value: *self_voi.value(),
                });
            }
        }
        self.values_over_intervals = new_voi;
        self.values_over_intervals.push(element);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::SmallPiecewiseBuilder;
    use crate::ValueOverInterval;
    use intervals_general::bound_pair::BoundPair;
    use intervals_general::interval::Interval;

    #[test]
    fn value_over_interval_mul() {
        let value_over_interval = ValueOverInterval::new(
            Interval::Closed {
                bound_pair: BoundPair::new(1.0, 2.0).unwrap(),
            },
            4,
        );

        let scaled_value = value_over_interval * 12;

        assert_eq!(
            *scaled_value.interval(),
            Interval::Closed {
                bound_pair: BoundPair::new(1.0, 2.0).unwrap(),
            }
        );
        assert_eq!(*scaled_value.value(), 48);
    }

    #[test]
    fn builder_add() {
        let mut builder: SmallPiecewiseBuilder<u32, f32> = SmallPiecewiseBuilder::new();
        builder.add(ValueOverInterval::new(Interval::Unbounded, 5.0));
        builder.add(ValueOverInterval::new(
            Interval::UnboundedClosedLeft { left: 230 },
            2.0,
        ));
        builder.add(ValueOverInterval::new(
            Interval::UnboundedOpenRight { right: 200 },
            1.0,
        ));
        let small_piecewise = builder.build();

        println!("{}", small_piecewise);

        assert_eq!(small_piecewise.value_at(1), Some(&1.0));
        assert_eq!(small_piecewise.value_at(210), Some(&5.0));
        assert_eq!(small_piecewise.value_at(230), Some(&2.0));
        assert_eq!(small_piecewise.value_at(231), Some(&2.0));
    }
}
