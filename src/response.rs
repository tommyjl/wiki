use axum::response::IntoResponse;
use bytes::Bytes;
use http::{header, HeaderValue, Response};
use http_body::Full;
use std::convert::Infallible;

macro_rules! response_with_content_type {
    ($($id:ident $content_type:expr;)*) => (
        $(
            #[derive(Clone, Copy, Debug)]
            pub struct $id<T>(pub T);

            impl<T> IntoResponse for $id<T>
            where
                T: Into<Full<Bytes>>,
            {
                type Body = Full<Bytes>;
                type BodyError = Infallible;

                fn into_response(self) -> Response<Self::Body> {
                    let mut res = Response::new(self.0.into());
                    res.headers_mut().insert(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static($content_type),
                    );
                    res
                }
            }
        )*
    )
}

response_with_content_type! {
    Markdown "text/markdown";
    Css "text/css";
}
