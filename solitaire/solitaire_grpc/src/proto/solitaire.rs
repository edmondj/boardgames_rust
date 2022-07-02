#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Card {
    #[prost(enumeration="Suite", tag="1")]
    pub suite: i32,
    #[prost(uint32, tag="2")]
    pub rank: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Foundation {
    #[prost(enumeration="Suite", tag="1")]
    pub suite: i32,
    #[prost(uint32, optional, tag="2")]
    pub value: ::core::option::Option<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Tableau {
    #[prost(uint64, tag="1")]
    pub downfaced_len: u64,
    #[prost(message, repeated, tag="2")]
    pub upturned: ::prost::alloc::vec::Vec<Card>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct State {
    #[prost(uint32, tag="1")]
    pub draw_pile_size: u32,
    #[prost(message, optional, tag="2")]
    pub upturned: ::core::option::Option<Card>,
    #[prost(message, repeated, tag="3")]
    pub foundations: ::prost::alloc::vec::Vec<Foundation>,
    #[prost(message, repeated, tag="4")]
    pub tableaus: ::prost::alloc::vec::Vec<Tableau>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Action {
    #[prost(oneof="action::Action", tags="1, 2, 3")]
    pub action: ::core::option::Option<action::Action>,
}
/// Nested message and enum types in `Action`.
pub mod action {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Draw {
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct BuildFoundation {
        #[prost(oneof="build_foundation::Source", tags="1, 2")]
        pub source: ::core::option::Option<build_foundation::Source>,
    }
    /// Nested message and enum types in `BuildFoundation`.
    pub mod build_foundation {
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Upturned {
        }
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Tableau {
            #[prost(uint32, tag="1")]
            pub index: u32,
        }
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Source {
            #[prost(message, tag="1")]
            Upturned(Upturned),
            #[prost(message, tag="2")]
            Tableau(Tableau),
        }
    }
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct BuildTableau {
        #[prost(uint32, tag="3")]
        pub destination_index: u32,
        #[prost(oneof="build_tableau::Source", tags="1, 2")]
        pub source: ::core::option::Option<build_tableau::Source>,
    }
    /// Nested message and enum types in `BuildTableau`.
    pub mod build_tableau {
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Upturned {
        }
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Tableau {
            #[prost(uint32, tag="1")]
            pub index: u32,
            #[prost(uint32, tag="2")]
            pub size: u32,
        }
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Source {
            #[prost(message, tag="1")]
            Upturned(Upturned),
            #[prost(message, tag="2")]
            Tableau(Tableau),
        }
    }
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Action {
        #[prost(message, tag="1")]
        Draw(Draw),
        #[prost(message, tag="2")]
        BuildFoundation(BuildFoundation),
        #[prost(message, tag="3")]
        BuildTableau(BuildTableau),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateGameRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateGameResponse {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub state: ::core::option::Option<State>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DestroyGameRequest {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DestroyGameResponse {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActRequest {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(message, optional, tag="2")]
    pub action: ::core::option::Option<Action>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActResponse {
    #[prost(bool, tag="1")]
    pub victory: bool,
    #[prost(message, optional, tag="2")]
    pub state: ::core::option::Option<State>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WatchRequest {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WatchResponse {
    #[prost(message, optional, tag="1")]
    pub action: ::core::option::Option<Action>,
    #[prost(message, optional, tag="2")]
    pub state: ::core::option::Option<State>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Suite {
    Undefined = 0,
    Hearts = 1,
    Diamonds = 2,
    Clubs = 3,
    Spades = 4,
}
/// Generated server implementations.
pub mod solitaire_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with SolitaireServer.
    #[async_trait]
    pub trait Solitaire: Send + Sync + 'static {
        async fn create_game(
            &self,
            request: tonic::Request<super::CreateGameRequest>,
        ) -> Result<tonic::Response<super::CreateGameResponse>, tonic::Status>;
        async fn destroy_game(
            &self,
            request: tonic::Request<super::DestroyGameRequest>,
        ) -> Result<tonic::Response<super::DestroyGameResponse>, tonic::Status>;
        async fn act(
            &self,
            request: tonic::Request<super::ActRequest>,
        ) -> Result<tonic::Response<super::ActResponse>, tonic::Status>;
        ///Server streaming response type for the Watch method.
        type WatchStream: futures_core::Stream<
                Item = Result<super::WatchResponse, tonic::Status>,
            >
            + Send
            + 'static;
        async fn watch(
            &self,
            request: tonic::Request<super::WatchRequest>,
        ) -> Result<tonic::Response<Self::WatchStream>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct SolitaireServer<T: Solitaire> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Solitaire> SolitaireServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for SolitaireServer<T>
    where
        T: Solitaire,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/solitaire.Solitaire/CreateGame" => {
                    #[allow(non_camel_case_types)]
                    struct CreateGameSvc<T: Solitaire>(pub Arc<T>);
                    impl<
                        T: Solitaire,
                    > tonic::server::UnaryService<super::CreateGameRequest>
                    for CreateGameSvc<T> {
                        type Response = super::CreateGameResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateGameRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).create_game(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateGameSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/solitaire.Solitaire/DestroyGame" => {
                    #[allow(non_camel_case_types)]
                    struct DestroyGameSvc<T: Solitaire>(pub Arc<T>);
                    impl<
                        T: Solitaire,
                    > tonic::server::UnaryService<super::DestroyGameRequest>
                    for DestroyGameSvc<T> {
                        type Response = super::DestroyGameResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DestroyGameRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).destroy_game(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DestroyGameSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/solitaire.Solitaire/Act" => {
                    #[allow(non_camel_case_types)]
                    struct ActSvc<T: Solitaire>(pub Arc<T>);
                    impl<T: Solitaire> tonic::server::UnaryService<super::ActRequest>
                    for ActSvc<T> {
                        type Response = super::ActResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ActRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).act(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ActSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/solitaire.Solitaire/Watch" => {
                    #[allow(non_camel_case_types)]
                    struct WatchSvc<T: Solitaire>(pub Arc<T>);
                    impl<
                        T: Solitaire,
                    > tonic::server::ServerStreamingService<super::WatchRequest>
                    for WatchSvc<T> {
                        type Response = super::WatchResponse;
                        type ResponseStream = T::WatchStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::WatchRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).watch(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = WatchSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Solitaire> Clone for SolitaireServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Solitaire> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Solitaire> tonic::transport::NamedService for SolitaireServer<T> {
        const NAME: &'static str = "solitaire.Solitaire";
    }
}
