#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use anyhow::{ bail, Context, Result };
use std::env::current_dir;
use std::io::stderr;
use std::io::stdout;
use std::io::Write;
use std::num::NonZeroU32;
use std::pin::pin;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::ggml_time_us;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
// use llama_cpp_2::model::params::kv_overrides::ParamOverrideValue;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::AddBos;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::token::data_array::LlamaTokenDataArray;

pub fn run_mistral(prompt: String) -> Result<String> {
    // set the length of the prompt + output in tokens
    // let n_len = 32;
    // let short = &input[..1000];

    // let prompt = format!(
    //     "Your task is to create a list of actions for the strategy:\n\nMarketing brief:\n{short:?}...\n\nStrategy:\n\n"
    // );
    let n_len = prompt.len() as i32;

    // https://huggingface.co/TheBloke/airoboros-13b-gpt4-GGML/discussions/1

    let layers = 4096;
    let context_length = 2 * 2048;
    let llama_batch = 512;
    let seed = 4422;

    let model_name = "mistral-7b-instruct-v0.2.Q2_K.gguf";
    let model_path = current_dir().unwrap().join("models").join(model_name);

    let backend = LlamaBackend::init().unwrap();
    let model_params = { LlamaModelParams::default().with_n_gpu_layers(layers) };
    // let mut model_params = pin!(model_params);
    let model_params = pin!(model_params);

    // for (k, v) in &key_value_overrides {
    //     let k = CString::new(k.as_bytes())
    //         .with_context(|| format!("invalid key: {k}"))
    //         .unwrap();
    //     model_params.as_mut().append_kv_override(k.as_c_str(), *v);
    // }

    let model = LlamaModel::load_from_file(&backend, model_path, &model_params)
        .with_context(|| "unable to load model")
        .unwrap();

    // initialize the context
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(NonZeroU32::new(context_length))
        .with_seed(seed);

    let mut ctx = model
        .new_context(&backend, ctx_params)
        .with_context(|| "unable to create the llama_context")
        .unwrap();

    let tokens_list = model
        .str_to_token(&prompt, AddBos::Always)
        .with_context(|| format!("failed to tokenize {prompt}"))
        .unwrap();

    let n_cxt = ctx.n_ctx() as i32;
    let n_kv_req = (tokens_list.len() as i32) + (n_len - (tokens_list.len() as i32));

    // eprintln!("n_len = {n_len}, n_ctx = {n_cxt}, k_kv_req = {n_kv_req}");

    // make sure the KV cache is big enough to hold all the prompt and generated tokens
    if n_kv_req > n_cxt {
        bail!(
            "n_kv_req > n_ctx, the required kv cache size is not big enough
    either reduce n_len or increase n_ctx"
        );
    }

    if tokens_list.len() >= usize::try_from(n_len).unwrap() {
        bail!("the prompt is too long, it has more tokens than n_len");
    }

    // print the prompt token-by-token
    // eprintln!();

    // for token in &tokens_list {
    //     eprint!("{}", model.token_to_str(*token).unwrap());
    // }

    stderr().flush().unwrap();

    // create a llama_batch with size 512
    // we use this object to submit token data for decoding
    let mut batch = LlamaBatch::new(llama_batch, 1);

    let last_index: i32 = (tokens_list.len() - 1) as i32;
    for (i, token) in (0_i32..).zip(tokens_list.into_iter()) {
        // llama_decode will output logits only for the last token of the prompt
        let is_last = i == last_index;
        batch.add(token, i, &[0], is_last).unwrap();
    }

    ctx.decode(&mut batch)
        .with_context(|| "llama_decode() failed")
        .unwrap();

    // main loop

    let mut n_cur = batch.n_tokens();
    let mut n_decode = 0;

    // let t_main_start = ggml_time_us();

    let mut answer: String = "".to_owned();

    while n_cur <= n_len {
        // sample the next token
        {
            let candidates = ctx.candidates_ith(batch.n_tokens() - 1);

            let candidates_p = LlamaTokenDataArray::from_iter(candidates, false);

            // sample the most likely token
            // TODO: beam search instead?
            let new_token_id = ctx.sample_token_greedy(candidates_p);

            // is it an end of stream?
            if new_token_id == model.token_eos() {
                eprintln!();
                break;
            }

            let str = model.token_to_str(new_token_id).unwrap();
            // uncomment to print in real time
            print!("{}", &str);

            answer.push_str(str.as_str());
            stdout().flush().unwrap();

            batch.clear();
            batch.add(new_token_id, n_cur, &[0], true).unwrap();
        }

        n_cur += 1;

        ctx.decode(&mut batch)
            .with_context(|| "failed to eval")
            .unwrap();

        n_decode += 1;
    }

    // eprintln!("\n");

    println!("=====================");
    println!("{answer}");
    println!("=====================");

    Ok(answer)
}
