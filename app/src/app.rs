use leptos::prelude::*;
use leptos_meta::{Stylesheet, Title, provide_meta_context};
use leptos_mview::mview;
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

use crate::reader::Reader;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    mview! {
        Stylesheet href="/pkg/app.css";
        Title text="Literacy";
        Router {
            main {
                Routes fallback={|| "Page not found.".into_view()} {
                    Route path={StaticSegment("")} view={ReadingPage};
                }
            }
        }
    }
}

#[component]
fn ReadingPage() -> impl IntoView {
    mview! {
        Reader;
    }
}
