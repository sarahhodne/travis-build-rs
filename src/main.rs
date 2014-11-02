extern crate serialize;
extern crate travis_build;

use serialize::json;

fn main() {
    let payload_str = "{\"repository\":{\"slug\":\"henrikhodne/test\",\"source_url\":\"git://github.com/henrikhodne/test.git\"},\"job\":{\"commit\":\"abcdef\",\"branch\":\"master\",\"pull_request\":false},\"config\":{\"language\":\"rust\",\"os\":\"linux\"}}";
    let payload_json = match json::from_str(payload_str) {
        Ok(p) => p,
        Err(e) => panic!("couldn't parse JSON: {}", e)
    };
    let payload = match travis_build::Payload::from_json(&payload_json) {
        Ok(p) => p,
        Err(e) => panic!("couldn't parse JSON to payload: {}", e)
    };

    print!("{}", travis_build::Script::new(payload).to_script());
}
