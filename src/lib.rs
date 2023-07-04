#![feature(return_position_impl_trait_in_trait)]

use std::io::Error;

use futures::Future;
use mockall::automock;

#[automock]
pub trait Foo {
    fn bar(&self) -> impl Future<Output = Result<i32, Error>>;
}

pub trait BazT {
    fn baz(&self) -> impl Future<Output = Result<i32, Error>> + '_;
}

pub struct Baz<F> {
    foo: F,
}

impl<F> Baz<F>
where
    F: Foo,
{
    pub fn new(foo: F) -> Self {
        Self { foo }
    }
}

impl<F> BazT for Baz<F>
where
    F: Foo,
{
    fn baz(&self) -> impl Future<Output = Result<i32, Error>> + '_ {
        async { self.foo.bar().await }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        pin::Pin,
        task::{Context, Poll},
    };

    use rand::random;

    use super::*;

    pub struct Foo1 {}

    impl Foo for Foo1 {
        fn bar(&self) -> impl Future<Output = Result<i32, Error>> {
            async {
                let x: u8 = random();
                if x > 100 {
                    Ok(1)
                } else {
                    Err(Error::from_raw_os_error(12))
                }
            }
        }
    }

    #[test]
    fn test_framework() {
        let baz = Baz::new(Foo1 {});
        futures::executor::block_on(async move {
            assert_eq!(Some(1), baz.baz().await.ok());
        });
    }

    /// Mock Foo
    pub struct Bar {
        value: i32,
    }

    impl Future for Bar {
        type Output = Result<i32, Error>;

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.value >= 0 {
                Poll::Ready(Ok(self.value))
            } else {
                Poll::Ready(Err(Error::from_raw_os_error(12)))
            }
        }
    }

    #[test]
    fn test_with_mock() {
        let mut mock = MockFoo::new();
        mock.expect_bar().returning(|| Box::pin(Bar { value: 2 }));
        let baz = Baz::new(mock);
        futures::executor::block_on(async move {
            assert_eq!(2, baz.baz().await.unwrap());
        });
    }
}
