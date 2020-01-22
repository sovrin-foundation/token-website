#[macro_use]
extern crate trace_macros;

use isahc::prelude::*;
use serde::{Serialize, Deserialize};
use sha2::Digest;
use sodiumoxide::crypto::sign::{
    sign_detached, gen_keypair,
    ed25519::SecretKey
};
use structopt::StructOpt;
use web_view::*;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "basic",
    version = "0.1",
    about = "Sovrin Foundation Token Website"
)]
struct Opt {
    #[structopt(subcommand)]
    pub cmd: Command
}

#[derive(Debug, StructOpt)]
enum Command {
    #[structopt(name = "sign")]
    Sign {
        #[structopt(short, long)]
        key: Option<String>,
        #[structopt(name = "TOKEN")]
        token: String
    }
}

#[derive(Serialize)]
struct PaymentAddressChallengeReponse {
    address: String,
    challenge: String,
    signature: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WebCmd {
    consents: String,
    data: String,
    path: String,
    verb: String,
    url: String,
}

const MAIN_PAGE_1: &str = r##"
<html>
<head lang="en">
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<link rel="stylesheet" href="https://stackpath.bootstrapcdn.com/bootstrap/4.4.1/css/bootstrap.min.css" integrity="sha384-Vkoo8x4CGsO3+Hhxv8T/Q5PaXtkKtu6ug5TOeNV6gBiFeWPGFN9MuhOf23Q9Ifjh" crossorigin="anonymous">
<style>
.table-row {
    margin: 0px 0px 15px 0px;
}
</style>
</head>
<body>
<div class="container-fluid">
    <div class="row">
        <div class="col-md-12">&nbsp;</div>
    </div>
    <div class="row table-row">
        <div class="col-md-12"><span id="error" style="color:red;"></span></div>
    </div>
    <div class="row table-row">
        <div class="col-md-2"><strong>URL</strong></div>
        <div class="col-md-10"><input id="test_url" type="url" placeholder="https://127.0.0.1:8000/api/v1" width="400px"></div>
    </div>
    <div class="row table-row">
        <div class="col-md-2"><strong>Paths</strong></div>
        <div class="col-md-4"><select id="paths">
        <option value="countries">countries</option>
        <option value="consents">consents</option>
        <option value="payment_address_challenge">payment_address_challenge</option>
        </select></div>
        <div class="col-md-6"><select id="consent_countries">
"##;        
const MAIN_PAGE_2: &str = r##"
</select></div>
    </div>
    <div class="row table-row" style="margin:0px 0px 20px 0px;">
        <div class="col-md-2"><strong>Commands</strong></div>
        <div class="col-md-2"><select id="verbs">
        <option value="get">GET</option>
        </select></div>
        <div class="col-md-8"><p><strong id="verb_data_label"></strong></p><textarea id="verb_data" type="text" width="100%" height="100%"></textarea></div>
    </div>
    <div class="row table-row">
        <div class="col-md-12"><button onclick="return perform_action();" width="30px" heigth="30px">Send</button></div>
    </div>
    <div class="row table-row">
        <div class="col-md-2"><strong>Request</strong></div>
        <div class="col-md-10"><textarea id="request" readonly width="100%" height="100%"></textarea></div>
    </div>
    <div class="row table-row">
        <div class="col-md-2"><strong>Response</strong></div>
        <div class="col-md-10"><textarea id="response" readonly width="100%" height="100%"></textarea></div>
    </div>
</div>
<script src="https://code.jquery.com/jquery-3.4.1.slim.min.js" integrity="sha384-J6qa4849blE2+poT4WnyKhv5vZF5SrPo0iEjwBvKU7imGFAV0wwj1yYfoRSJoZ+n" crossorigin="anonymous"></script>
<script src="https://cdn.jsdelivr.net/npm/popper.js@1.16.0/dist/umd/popper.min.js" integrity="sha384-Q6E9RHvbIyZFJoft+2mJbHaEWldlvI9IOYy5n3zV9zzTtmI3UksdQRVvoxMfooAo" crossorigin="anonymous"></script>
<script src="https://stackpath.bootstrapcdn.com/bootstrap/4.4.1/js/bootstrap.min.js" integrity="sha384-wfSDF2E50Y2D1uUdj0O3uMBJnjuUD4Ih7YwaYd1iqfktj0Uod8GCExl3Og8ifwB6" crossorigin="anonymous"></script>
<script type="text/javascript">
    function perform_action() {
        var o = new Object();
        o.url = $('#test_url').val();
        o.path = $('#paths option:selected').val();
        o.verb = $('#verbs option:selected').text();
        o.consents = $('#consent_countries option:selected').val();
        switch (o.verb) {
            case "post":
            case "put":
                o.data = $('#verb_data').val();
                break;
            default:
                o.data = "";
                break;
        }
        window.external.invoke(JSON.stringify(o));
    }

    function set_error(message) {
        $('#error').html(message);
    }

    function result_returned(request, response) {
        $('#request').val(request);
        $('#response').val(response);
    }

    $(document).ready(function() {
        $('#verb_data').hide();
        $('#consent_countries').hide();
        $('#verbs').change(function() {
            var selectedVerb = $(this).children("option:selected").val();

            switch (selectedVerb) {
                case "get":
                case "delete":
                    $('#verb_data').hide();
                    break;
                case "post":
                case "put":
                    $('#verb_data').show();
                    $('#verb_data_label').html(selectedVerb.toUpperCase() + " Body Data");
                    break;
                default:
                    $('#verb_data').hide();
                    break;
            }
        });

        $('#paths').change(function() {
            var selectedPath = $(this).children("option:selected").val();

            switch (selectedPath) {
                case "countries":
                    $('#verbs').empty().append('<option selected="selected" value="get">GET</option>');
                    $('#consent_countries').hide();
                    break;
                case "consents":
                    $('#verbs').empty().append('<option selected="selected" value="get">GET</option>');
                    $('#consent_countries').show();
                    break;
                case "payment_address_challenge":
                    $('#verbs').empty().append('<option selected="selected" value="get">GET</option><option value="post">POST</option>');
                    $('#consent_countries').hide();
                    break;
                default:
                    $('#verbs').empty();
                    $('#consent_countries').hide();
                    break;
            }
        });
    });
</script>
</body>
</html>
"##;


fn main() {
    // let (tx, rx) = std::sync::mpsc::channel::<WebCmd>();
    // let mut countries = celes::Country::get_countries().iter().map(|c| format!("<option value=\"{}\">{} - {}</option>", c.alpha2, c.alpha2, c.long_name)).collect::<Vec<String>>();
    // countries.sort();
    // let mut page = String::new();
    // page.push_str(MAIN_PAGE_1);
    // page.push_str(&countries.join(""));
    // page.push_str(MAIN_PAGE_2);
    // let webview = web_view::builder()
    //     .title("Website API tester")
    //     .content(Content::Html(page))
    //     .user_data(0)
    //     .size(1200, 800)
    //     .resizable(true)
    //     .invoke_handler(|_, arg| {
    //         let cmd = serde_json::from_str(arg).unwrap();
    //         tx.send(cmd).unwrap();
    //         Ok(())
    //     }).build().unwrap();
    // let wv_handle = webview.handle();
    // std::thread::spawn(move ||{
    //     loop {
    //         let cmd = rx.recv().unwrap();
    //         info!("{:?}", cmd);

    //         if cmd.clone().url.is_empty() {
    //             wv_handle.dispatch(|wv| {
    //                 wv.eval(&format!(r##"set_error("URL cannot be empty");"##))
    //             }).unwrap();
    //             continue;
    //         }

    //         let mut request_url = format!("{}/{}", cmd.url, cmd.path);
    //         if cmd.consents.len() > 0 {
    //             request_url.push_str("/");
    //             request_url.push_str(&cmd.consents);
    //         }
    //         let send_body = cmd.clone().data;
    //         let request = Request::builder()
    //                             .uri(request_url.to_string())
    //                             .method(cmd.clone().verb.as_str())
    //                             .header("Accept", "application/json")
    //                             .body(send_body).map_err(|e| format!("{:?}", e));

    //         let mut request_text = format!(r##"{}  {}"##, cmd.verb, request_url);
    //         if !cmd.clone().data.is_empty() {
    //             request_text.push_str("\n\n");
    //             request_text.push_str(&cmd.data);
    //         }

    //         let mut response = match request {
    //             Ok(r) => match r.send() {
    //                 Ok(res) => res,
    //                 Err(e) => {
    //                     wv_handle.dispatch(move |wv| {
    //                         wv.eval(&format!(r##"set_error("{}");"##, e))
    //                     }).unwrap();
    //                     continue;
    //                 }
    //             },
    //             Err(e) => {
    //                 wv_handle.dispatch(move |wv| {
    //                     wv.eval(&format!(r##"set_error("{}");"##, e))
    //                 }).unwrap();
    //                 continue;
    //             }
    //         };

    //         let response_text = response.text();

    //         let body = match response_text {
    //             Err(e) => {
    //                 wv_handle.dispatch(move |wv| {
    //                     wv.eval(&format!(r##"set_error("{}");"##, e))
    //                 }).unwrap();
    //                 continue;
    //             },
    //             Ok(b) => b,
    //         };

    //         info!("request_text = {:?}", request_text);
    //         info!("response_text = {:?}", body);

    //         wv_handle.dispatch(move |wv| {
    //             wv.eval(&format!(r##"result_returned('{}', '{}');"##, request_text, body))
    //         }).unwrap();
    //     }
    // });
    // webview.run().unwrap();


    let opt = Opt::from_args();
    match opt.cmd {
        Command::Sign { key, token } => {
            let (pk, sk) = match key {
                Some(k) => {
                    let k1 = bs58::decode(k).into_vec().unwrap();
                    let sk1 = SecretKey::from_slice(k1.as_slice()).unwrap();
                    let pk1 = sk1.public_key();
                    (pk1, sk1)
                },
                None => gen_keypair()
            };
            let mut sha = sha2::Sha256::new();

            let challenge = base64_url::decode(&token).unwrap();

            sha.input(format!("\x6DSovrin Signed Message:\nLength: {}\n", challenge.len()).as_bytes());
            sha.input(challenge.as_slice());
            let data = sha.result();
            let signature = sign_detached(data.as_slice(), &sk);

            let response = PaymentAddressChallengeReponse {
                address: format!("pay:sov:{}", bs58::encode(&pk[..]).with_check().into_string()),
                challenge: token,
                signature: base64_url::encode(&signature[..])
            };

            println!("key = {}", bs58::encode(sk).with_check().into_string());
            println!("response = {}", serde_json::to_string(&response).unwrap());
        }
    }
}
