/*
    COMPREHENSION: MAPPING FOR_IF_CLAUSE

    MAPPING: EXPR

    FOR_IF_CLAUSE: "for" PATTERN "in" EXPR ("if" EXPR)* 

    PATTERN: name (, name)*
*/

use std::{ops::{Deref, DerefMut}, str::FromStr};

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse_macro_input, spanned::Spanned};

// DATA DEFINITION

struct Comprehension {
    mapping: Mapping,
    for_if_clause: ForIfClause
}

struct Mapping(syn::Expr);

struct ForIfClause {
    pattern: Pattern,
    iterable_expression: syn::Expr,
    conditions: Vec<Condition>
}

struct Pattern(syn::Pat);
struct Condition(syn::Expr);

// IMPLEMENTATION FROM TOKENS TO DATA

// Parse streams are essentially an iterable of tokens

impl syn::parse::Parse for Pattern {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        syn::Pat::parse_single(input).map(Self)
    }
}

impl syn::parse::Parse for Condition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // consume "if" token first, and explicitly discard it
        _ = input.parse::<syn::Token![if]>()?;
        input.parse().map(Self)
    }
}

impl syn::parse::Parse for Mapping {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse().map(Self)
    }
}

impl syn::parse::Parse for ForIfClause {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // consume "for" token
        _ = input.parse::<syn::Token![for]>()?;
        let pattern: Pattern = input.parse()?;

        // consume "in"
        _ = input.parse::<syn::Token![in]>()?;
        let iterable_expression: syn::Expr = input.parse()?;

        let conditions: Vec<Condition> = parse_zero_or_more::<Condition>(input);

        Ok(Self {
            pattern, iterable_expression, conditions
        })
    }
}

fn parse_zero_or_more<T: syn::parse::Parse>(input: syn::parse::ParseStream) -> Vec<T> {
    let mut result = vec![];

    while let Ok(item) = input.parse() {
        result.push(item);
    }

    return result;
}

impl syn::parse::Parse for Comprehension {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // this feels like magic how the fuck is this code that works
        Ok(Comprehension {
            mapping: input.parse()?, 
            for_if_clause: input.parse()?
        })
    }
}

// IMPLEMENTATION FROM DATA TO RUST

impl quote::ToTokens for Pattern {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl quote::ToTokens for Condition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens)
    }
}

impl quote::ToTokens for Comprehension {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        // ::core::iter::IntoIterator::into_iter(sequence).filter_map(move |pattern| { (true && ...).then(|| mapping) })

        let Mapping(mapping) = &self.mapping;
        let ForIfClause{pattern, iterable_expression, conditions} = &self.for_if_clause;

        tokens.extend(quote::quote! {
            ::core::iter::IntoIterator::into_iter(#iterable_expression).filter_map(move |#pattern| {
                (true #(&& #conditions)*).then(|| #mapping)
            })
        });
    }
}

#[proc_macro]
pub fn comp(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let c = parse_macro_input!(input as Comprehension);
    quote::quote! { #c }.into()
}

// Now for something a bit harder -- can I make a proc macro that transpiles
// (a subset of) C to Rust?

// okay this is way harder than I thought

struct CFunction {
    return_type: syn::Type,
    function_name: syn::Ident,
    function_params: ParameterList,
    function_body: FunctionBody
}

struct ParameterList {
    paren_token_open: syn::token::Paren,
    parameters: syn::punctuated::Punctuated<Parameter, syn::Token![,]>
}

struct Parameter {
    param_type: syn::Type,
    param_name: syn::Ident
}

struct FunctionBody {
    opening_brace: syn::token::Brace,
    statements: syn::punctuated::Punctuated<syn::Expr, syn::Token![;]>
}

impl syn::parse::Parse for Parameter {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let param_type: syn::Type = c_to_rust_type(input)?;
        let param_name: syn::Ident = input.parse()?;

        Ok(Self {
            param_type,
            param_name
        })
    }
}

impl syn::parse::Parse for ParameterList {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;

        Ok(Self {
            paren_token_open: syn::parenthesized!(content in input),
            parameters: content.parse_terminated(Parameter::parse, syn::Token![,])?
        })
    }
}

impl syn::parse::Parse for FunctionBody {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;

        Ok(Self {
            opening_brace: syn::braced!(content in input),
            statements: content.parse_terminated(syn::Expr::parse, syn::Token![;])?
        })
    }
}

impl syn::parse::Parse for CFunction {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let return_type = c_to_rust_type(input)?;
        let function_name: syn::Ident = input.parse()?;
        let function_params: ParameterList = input.parse()?;
        let function_body: FunctionBody = input.parse()?;

        Ok(Self {
            return_type,
            function_name,
            function_params,
            function_body
        })
    }
}

fn c_to_rust_type(input: syn::parse::ParseStream) -> syn::Result<syn::Type> {
    if let Ok(typestr) = input.parse::<syn::Ident>() {
        match typestr.to_string().as_str() {
            "int" =>    return Ok(syn::Type::Verbatim(TokenStream::from_str("i32")?)),
            "void" =>   return Ok(syn::Type::Verbatim(TokenStream::from_str("()")?)),
            _ =>        return Err(syn::Error::new(input.span(), "Only void and int supported"))
        }
    }

    Err(syn::Error::new(input.span(), "Unable to convert token to String"))
}

impl quote::ToTokens for Parameter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let param_type = &self.param_type;
        let param_name = &self.param_name;

        tokens.extend(quote::quote! {
            #param_name: #param_type
        })
    }
}

impl quote::ToTokens for ParameterList {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let params = &self.parameters;

        tokens.extend(quote::quote! {
            (#params)
        })
    }
}

// TODO currently this doesn't do any transpilation on the statements themselves,
// meaning that the function body will only accept valid Rust code
// There's a good amount of overlap between the two languages, but not enough
// to get any meaningful work out of it
impl quote::ToTokens for FunctionBody {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let statements = &mut self.statements.clone();

        let mut transpiled_statements = Vec::new();

        // This feels absolutely disgusting.
        for statement in statements {
            match statement {
                syn::Expr::Call(expr_call) => {
                    match expr_call.func.as_ref() {
                        syn::Expr::Path(expr_path) => {
                            if let Some(ident) = &expr_path.path.segments.first() {
                                if ident.ident == "printf" {
                                    let args = &expr_call.args;
                                    let new_expr = syn::parse_str::<syn::Expr>(
                                        &quote::quote! { print!(#args) }
                                        .to_string()
                                    ).unwrap();
    
                                    // println!("New expression: {:?} ({:?})", new_expr, new_expr.to_token_stream());
    
                                    transpiled_statements.push(new_expr);
                                } else {
                                    transpiled_statements.push(statement.clone())
                                }
                            }

                        },
                        _ => transpiled_statements.push(statement.clone()),
                    }                    
                },
                _ => transpiled_statements.push(statement.clone()),
            }
        }

        tokens.extend(quote::quote! {
            {
                #(#transpiled_statements;)*
            }
        })
    }
}

impl quote::ToTokens for CFunction {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {return_type, function_name, function_params, function_body} = &self;

        tokens.extend(quote::quote! {
            fn #function_name #function_params -> #return_type
            #function_body
        });
    }
}

#[proc_macro]
pub fn cfunc(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let c = parse_macro_input!(input as CFunction);
    quote::quote! { #c }.into()
}