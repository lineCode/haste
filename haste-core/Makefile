proto:
	cd src/protos/agent && protoc --rust_out=. --grpc_out=. --plugin=protoc-gen-grpc=`which grpc_rust_plugin` agent.proto && cd -
