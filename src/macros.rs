#![macro_use]

macro_rules! qs {
    ($($name:ident => $value:expr),+ $(),*) => {
        ::url::form_urlencoded::serialize([
            $((stringify!($name), $value)),*
        ].into_iter().filter(|&&(_, v)| !v.is_empty()))
    }
}

macro_rules! expand_value_expr {
    ($this:ident; $param_name:ident;) => {
        expand_value_expr!($this; $param_name; ToString)
    };
    ($this:ident; $param_name:ident; ToString) => {
        expand_value_expr!($this; $param_name; |value|
                           &*value.to_string())
    };
    ($this:ident; $param_name:ident; Borrow) => {
        expand_value_expr!($this; $param_name; |value|
                           value.borrow())
    };
    ($this:ident; $param_name:ident; AsRef) => {
        expand_value_expr!($this; $param_name; |value|
                           value.as_ref())
    };
    ($this:ident; $param_name:ident; bool) => {
        expand_value_expr!($this; $param_name; |value|
                           if *value {"1"} else {"0"})
    };
    ($this:ident; $param_name:ident; Vec) => {
        expand_value_expr!($this; $param_name; |value|
                           &*value.iter().map(ToString::to_string).collect::<Vec<_>>().join(","))
    };
    ($this:ident; $param_name:ident; Option) => {
        expand_value_expr!($this; $param_name; |value|
                           value.as_ref().map(ToString::to_string).as_ref()
                           .map(Borrow::borrow).unwrap_or(""))
    };
    ($this:ident; $param_name:ident; |$value_name:ident| $value_expr:expr) => {
        { let $value_name = &$this.$param_name; $value_expr }
    };
}

macro_rules! expand_init_expr {
    () => { Default::default() };
    ($value:expr) => { $value };
}

macro_rules! request_builder_impl {
    (
        $struct_name:ident (
            $($req_param_name:ident: $req_param_type:ty),*
        )
        {
            $($param_name:ident: $param_type:ty [$($param_value:tt)*]),*
            $(,)*
        }
    ) => {
        #[allow(non_camel_case_types)]
        pub fn new<$($req_param_name: Into<$req_param_type>),*>($($req_param_name: $req_param_name),*) -> Self {
            $struct_name {
                $($req_param_name: $req_param_name.into(),)*
                $($param_name: expand_init_expr!($($param_value)*),)*
            }
        }
        $(request_builder_setter_impl!($param_name: $param_type [$($param_value)*]);)*
    }
}

macro_rules! request_builder_setter_impl {
    (
        $param_name:ident: $param_type:ty [{$($param_value:tt)*}]
    ) => {
        pub fn $param_name<T: Into<$param_type>>(&mut self, value: T) -> &mut Self {
            self.$param_name = value.into();
            self
        }
    };
    (
        $param_name:ident: $param_type:ty [$($param_value:tt)*]
    ) => {
        pub fn $param_name(&mut self, value: $param_type) -> &mut Self {
            self.$param_name = value;
            self
        }
    };
}

macro_rules! request_trait_impl {
    (
        [$method_name:expr]($($const_param_name:ident => $const_param_value:expr),*) -> $response_type:ty
        {
            $($param_name:ident => {$($value:tt)*}),*
            $(,)*
        }
    ) => {
        type Response = $response_type;
        fn method_name() -> &'static str { $method_name }
        fn to_query_string(&self) -> String {
            qs![
                $($param_name => expand_value_expr!(self; $param_name; $($value)*),)*
                $($const_param_name => $const_param_value,)*
            ]
        }
    }
}

macro_rules! request {
    (
        $(#[$attr:meta])*
        struct $struct_name:ident (
            $($req_param_name:ident: $req_param_type:ty => {$($req_value:tt)*}),*
        ) for [$method_name:expr]
        ($($const_param_name:ident => $const_param_value:expr),*) ->
        $response_type:ty
        {
            $($param_name:ident: $param_type:ty [$($param_value:tt)*] => {$($value:tt)*}),*
            $(,)*
        }
    ) => {
        #[derive(Debug, PartialEq, Clone)]
        $(#[$attr])*
        pub struct $struct_name {
            $($param_name: $param_type,)*
            $($req_param_name: $req_param_type,)*
        }

        impl ::api::Request for $struct_name {
            request_trait_impl! {
                [$method_name]
                ($($const_param_name => $const_param_value),*)
                -> $response_type
                {
                    $($req_param_name => {$($req_value)*},)*
                    $($param_name =>  {$($value)*},)*
                }
            }
        }

        impl $struct_name {
            request_builder_impl! {
                $struct_name (
                    $($req_param_name: $req_param_type),*
                )
                {
                    $($param_name: $param_type [$($param_value)*]),*
                }
            }
        }
    };
}

