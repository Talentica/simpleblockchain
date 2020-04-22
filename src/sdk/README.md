## SDK Documentation

Use this SDK to develop, build and deploy the blockchain applications 

#### Steps to develop app
* Clone sdk directory locally
* Create RUST project with library type cdylib
* The app project must implement the entry point function
``` rust
#[no_mangle]
pub fn register_app() -> Box<dyn AppHandler + Send>
```
* App must implement ``` AppHandler ``` trait. 
```rust
pub trait AppHandler {
    fn execute(&self, txn: &SignedTransaction, state_context: &mut dyn StateContext) -> bool;
    fn name(&self) -> String;
}
````
* Add the transaction business validations inside the ``` execute ``` function.
* Please refer to [transaction file](../user/wallet_app/src/transaction.rs) in the wallet app.
* Cargo build should produce shared library output

#### Steps to deploy app
* Get node binary executable (either build it or use docker image)
* Update config.toml file and add application shared libray path under ```client_apps``` section
* Run node file, check log file entries to verify that the application binary files are getting loaded sucessfully
