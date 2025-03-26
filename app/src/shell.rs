use leptos::{config::LeptosOptions, prelude::*};
use leptos_meta::MetaTags;
use leptos_mview::mview;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    mview! {
        {view!{<!DOCTYPE html>}}
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                AutoReload options={options.clone()};
                HydrationScripts options={options.clone()};
                MetaTags;
            }
            body {
                crate::App;
            }
        }
    }
}
