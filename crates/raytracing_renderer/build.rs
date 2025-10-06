use cuda_builder::CudaBuilder;

fn main() {
    CudaBuilder::new("../raytracing_cuda_kernel")
        .copy_to("kernel.ptx")
        .build()
        .unwrap();
}