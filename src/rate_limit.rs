use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::stream::Stream;
use tokio::time::{interval, Interval, MissedTickBehavior};

const fn assert_stream<T, S>(stream: S) -> S
where
    S: Stream<Item = T>,
{
    stream
}

fn limiter(per_sec: u64) -> Interval {
    let delay_ms = ((1.0 / per_sec as f64) * 1_000.0) as u64;
    let mut limiter = interval(Duration::from_millis(delay_ms));
    limiter.set_missed_tick_behavior(MissedTickBehavior::Delay);
    limiter
}

pub struct RateLimitIter<I: Unpin> {
    iter: I,
    timer: Interval,
}

pub fn rate_limit<I>(i: I, per_sec: u64) -> RateLimitIter<I::IntoIter>
where
    I: IntoIterator,
    I::IntoIter: Unpin,
{
    assert_stream::<I::Item, _>(RateLimitIter {
        iter: i.into_iter(),
        timer: limiter(per_sec),
    })
}

impl<I: Unpin> RateLimitIter<I> {
    pub fn set_missed_tick_behavior(&mut self, opt: MissedTickBehavior) {
        self.timer.set_missed_tick_behavior(opt);
    }
}

impl<I> Stream for RateLimitIter<I>
where
    I: Iterator + Unpin,
{
    type Item = I::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<I::Item>> {
        match self.timer.poll_tick(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(_) => Poll::Ready(self.iter.next()),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[cfg(test)]
mod tests {
    use super::rate_limit;
    use futures::StreamExt;

    macro_rules! assert_elapsed_ms {
        ($now:ident, $ms_min:literal, $ms_max:literal) => {
            #[allow(unused_comparisons)]
            {
                let elapsed = $now.elapsed().as_millis();
                assert!(
                    elapsed >= $ms_min,
                    "rate limit ticked too fast ({}ms < {}ms)",
                    elapsed,
                    $ms_min
                );
                assert!(
                    elapsed <= $ms_max,
                    "rate limit ticked too slow ({}ms > {}ms)",
                    elapsed,
                    $ms_max
                );
            }
        };
    }

    #[tokio::test]
    async fn it_works() {
        let now = std::time::Instant::now();
        let mut count = rate_limit(0..4, 4);

        let _ = count.next().await;
        assert_elapsed_ms!(now, 0, 10);

        let _ = count.next().await;
        assert_elapsed_ms!(now, 240, 260);

        let _ = count.next().await;
        assert_elapsed_ms!(now, 490, 510);

        let _ = count.next().await;
        assert_elapsed_ms!(now, 740, 760);

        assert!(count.next().await.is_none());
        assert_elapsed_ms!(now, 990, 1010);
    }
}
