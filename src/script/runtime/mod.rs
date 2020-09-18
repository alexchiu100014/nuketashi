pub mod savedata;

use rusty_v8 as v8;
use v8::FunctionCallback;
use v8::MapFnTo;

fn hello_from_rust(
    scope: &mut v8::HandleScope,
    _: v8::FunctionCallbackArguments,
    mut retval: v8::ReturnValue,
) {
    log::debug!("script runtime started from: {:?}", std::thread::current());
    retval.set(v8::undefined(scope).into());
}

pub fn init() {
    let platform = v8::new_default_platform().unwrap();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let isolate = &mut v8::Isolate::new(Default::default());

    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);

    let func = v8::Function::new(scope, hello_from_rust).unwrap();
    let global = context.global(scope);
    let key = v8::String::new(scope, "helloFromRust").unwrap();
    global.set(scope, key.into(), func.into());

    let code = v8::String::new(scope, "helloFromRust()").unwrap();

    let script = v8::Script::compile(scope, code, None).unwrap();
    let _ = script.run(scope).unwrap();
}
