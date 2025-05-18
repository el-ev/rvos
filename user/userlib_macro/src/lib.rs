extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn user_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    let main_fn = &input.sig.ident;

    let expanded = quote! {
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        #[unsafe(link_section = ".text.entry")]
        pub unsafe extern "C" fn _start() -> ! {
            unsafe {
                core::arch::naked_asm!(
                    "
                    call {main}
                    ",
                    main = sym #main_fn,
                )
            }
        }

        #input
    };

    TokenStream::from(expanded)
}
