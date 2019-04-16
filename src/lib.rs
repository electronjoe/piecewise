use intervals_general::interval::Interval;
use itertools::iproduct;
use smallvec::smallvec;

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
    T: std::ops::Sub,
{
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
    T: std::ops::Sub,
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
            new_voi.push(ValueOverInterval {
                interval: self_voi.interval().intersect(&complement_interval),
                value: *self_voi.value(),
            });
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
