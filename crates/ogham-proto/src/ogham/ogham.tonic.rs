// @generated
/// Generated client implementations.
pub mod test_api_client {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    ///
    #[derive(Debug, Clone)]
    pub struct TestApiClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl TestApiClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> TestApiClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::Body>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> TestApiClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::Body>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::Body>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::Body>,
            >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
        {
            TestApiClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        ///
        pub async fn test_contract(
            &mut self,
            request: impl tonic::IntoRequest<super::Test>,
        ) -> std::result::Result<tonic::Response<super::Test>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ogham.TestAPI/TestContract",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ogham.TestAPI", "TestContract"));
            self.inner.unary(req, path, codec).await
        }
        ///
        pub async fn test_stream_contract(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::Test>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::Test>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ogham.TestAPI/TestStreamContract",
            );
            let mut req = request.into_streaming_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ogham.TestAPI", "TestStreamContract"));
            self.inner.streaming(req, path, codec).await
        }
        ///
        pub async fn test_bidirectional_stream_contract(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::Test>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::Test>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ogham.TestAPI/TestBidirectionalStreamContract",
            );
            let mut req = request.into_streaming_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ogham.TestAPI", "TestBidirectionalStreamContract"),
                );
            self.inner.streaming(req, path, codec).await
        }
        ///
        pub async fn test_client_stream_contract(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::Test>,
        ) -> std::result::Result<tonic::Response<super::Test>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ogham.TestAPI/TestClientStreamContract",
            );
            let mut req = request.into_streaming_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ogham.TestAPI", "TestClientStreamContract"));
            self.inner.client_streaming(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod test_api_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with TestApiServer.
    #[async_trait]
    pub trait TestApi: std::marker::Send + std::marker::Sync + 'static {
        ///
        async fn test_contract(
            &self,
            request: tonic::Request<super::Test>,
        ) -> std::result::Result<tonic::Response<super::Test>, tonic::Status>;
        /// Server streaming response type for the TestStreamContract method.
        type TestStreamContractStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::Test, tonic::Status>,
            >
            + std::marker::Send
            + 'static;
        ///
        async fn test_stream_contract(
            &self,
            request: tonic::Request<tonic::Streaming<super::Test>>,
        ) -> std::result::Result<
            tonic::Response<Self::TestStreamContractStream>,
            tonic::Status,
        >;
        /// Server streaming response type for the TestBidirectionalStreamContract method.
        type TestBidirectionalStreamContractStream: tonic::codegen::tokio_stream::Stream<
                Item = std::result::Result<super::Test, tonic::Status>,
            >
            + std::marker::Send
            + 'static;
        ///
        async fn test_bidirectional_stream_contract(
            &self,
            request: tonic::Request<tonic::Streaming<super::Test>>,
        ) -> std::result::Result<
            tonic::Response<Self::TestBidirectionalStreamContractStream>,
            tonic::Status,
        >;
        ///
        async fn test_client_stream_contract(
            &self,
            request: tonic::Request<tonic::Streaming<super::Test>>,
        ) -> std::result::Result<tonic::Response<super::Test>, tonic::Status>;
    }
    ///
    #[derive(Debug)]
    pub struct TestApiServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> TestApiServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
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
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for TestApiServer<T>
    where
        T: TestApi,
        B: Body + std::marker::Send + 'static,
        B::Error: Into<StdError> + std::marker::Send + 'static,
    {
        type Response = http::Response<tonic::body::Body>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            match req.uri().path() {
                "/ogham.TestAPI/TestContract" => {
                    #[allow(non_camel_case_types)]
                    struct TestContractSvc<T: TestApi>(pub Arc<T>);
                    impl<T: TestApi> tonic::server::UnaryService<super::Test>
                    for TestContractSvc<T> {
                        type Response = super::Test;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::Test>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as TestApi>::test_contract(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = TestContractSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ogham.TestAPI/TestStreamContract" => {
                    #[allow(non_camel_case_types)]
                    struct TestStreamContractSvc<T: TestApi>(pub Arc<T>);
                    impl<T: TestApi> tonic::server::StreamingService<super::Test>
                    for TestStreamContractSvc<T> {
                        type Response = super::Test;
                        type ResponseStream = T::TestStreamContractStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<tonic::Streaming<super::Test>>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as TestApi>::test_stream_contract(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = TestStreamContractSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ogham.TestAPI/TestBidirectionalStreamContract" => {
                    #[allow(non_camel_case_types)]
                    struct TestBidirectionalStreamContractSvc<T: TestApi>(pub Arc<T>);
                    impl<T: TestApi> tonic::server::StreamingService<super::Test>
                    for TestBidirectionalStreamContractSvc<T> {
                        type Response = super::Test;
                        type ResponseStream = T::TestBidirectionalStreamContractStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<tonic::Streaming<super::Test>>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as TestApi>::test_bidirectional_stream_contract(
                                        &inner,
                                        request,
                                    )
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = TestBidirectionalStreamContractSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ogham.TestAPI/TestClientStreamContract" => {
                    #[allow(non_camel_case_types)]
                    struct TestClientStreamContractSvc<T: TestApi>(pub Arc<T>);
                    impl<T: TestApi> tonic::server::ClientStreamingService<super::Test>
                    for TestClientStreamContractSvc<T> {
                        type Response = super::Test;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<tonic::Streaming<super::Test>>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as TestApi>::test_client_stream_contract(&inner, request)
                                    .await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = TestClientStreamContractSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.client_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        let mut response = http::Response::new(
                            tonic::body::Body::default(),
                        );
                        let headers = response.headers_mut();
                        headers
                            .insert(
                                tonic::Status::GRPC_STATUS,
                                (tonic::Code::Unimplemented as i32).into(),
                            );
                        headers
                            .insert(
                                http::header::CONTENT_TYPE,
                                tonic::metadata::GRPC_CONTENT_TYPE,
                            );
                        Ok(response)
                    })
                }
            }
        }
    }
    impl<T> Clone for TestApiServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    /// Generated gRPC service name
    pub const SERVICE_NAME: &str = "ogham.TestAPI";
    impl<T> tonic::server::NamedService for TestApiServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
