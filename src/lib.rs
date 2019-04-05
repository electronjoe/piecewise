use intervals_general::interval::Interval;

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

#[cfg(test)]
mod tests {
    use crate::ValueOverInterval;
    use intervals_general::bound_pair::BoundPair;
    use intervals_general::interval::Interval;

    #[test]
    fn mul() {
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
}
