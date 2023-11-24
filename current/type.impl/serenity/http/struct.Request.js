(function() {var type_impls = {
"serenity":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Request%3C'a%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#34-148\">source</a><a href=\"#impl-Request%3C'a%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'a&gt; <a class=\"struct\" href=\"serenity/http/struct.Request.html\" title=\"struct serenity::http::Request\">Request</a>&lt;'a&gt;</h3></section></summary><div class=\"impl-items\"><section id=\"method.new\" class=\"method\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#35-44\">source</a><h4 class=\"code-header\">pub const fn <a href=\"serenity/http/struct.Request.html#tymethod.new\" class=\"fn\">new</a>(route: <a class=\"enum\" href=\"serenity/http/enum.Route.html\" title=\"enum serenity::http::Route\">Route</a>&lt;'a&gt;, method: <a class=\"enum\" href=\"serenity/http/enum.LightMethod.html\" title=\"enum serenity::http::LightMethod\">LightMethod</a>) -&gt; Self</h4></section><section id=\"method.body\" class=\"method\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#46-49\">source</a><h4 class=\"code-header\">pub fn <a href=\"serenity/http/struct.Request.html#tymethod.body\" class=\"fn\">body</a>(self, body: <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u8.html\">u8</a>&gt;&gt;) -&gt; Self</h4></section><section id=\"method.multipart\" class=\"method\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#51-54\">source</a><h4 class=\"code-header\">pub fn <a href=\"serenity/http/struct.Request.html#tymethod.multipart\" class=\"fn\">multipart</a>(self, multipart: <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"serenity/http/struct.Multipart.html\" title=\"struct serenity::http::Multipart\">Multipart</a>&gt;) -&gt; Self</h4></section><section id=\"method.headers\" class=\"method\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#56-59\">source</a><h4 class=\"code-header\">pub fn <a href=\"serenity/http/struct.Request.html#tymethod.headers\" class=\"fn\">headers</a>(self, headers: <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"https://docs.rs/http/0.2.11/http/header/map/struct.HeaderMap.html\" title=\"struct http::header::map::HeaderMap\">Headers</a>&gt;) -&gt; Self</h4></section><section id=\"method.params\" class=\"method\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#61-64\">source</a><h4 class=\"code-header\">pub fn <a href=\"serenity/http/struct.Request.html#tymethod.params\" class=\"fn\">params</a>(self, params: <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html\" title=\"struct alloc::vec::Vec\">Vec</a>&lt;(&amp;'static <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/string/struct.String.html\" title=\"struct alloc::string::String\">String</a>)&gt;&gt;) -&gt; Self</h4></section><section id=\"method.build\" class=\"method\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#66\">source</a><h4 class=\"code-header\">pub fn <a href=\"serenity/http/struct.Request.html#tymethod.build\" class=\"fn\">build</a>(\n    self,\n    client: &amp;<a class=\"struct\" href=\"https://docs.rs/reqwest/0.11.22/reqwest/async_impl/client/struct.Client.html\" title=\"struct reqwest::async_impl::client::Client\">Client</a>,\n    token: &amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>,\n    proxy: <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;&amp;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>&gt;\n) -&gt; <a class=\"type\" href=\"serenity/type.Result.html\" title=\"type serenity::Result\">Result</a>&lt;<a class=\"struct\" href=\"https://docs.rs/reqwest/0.11.22/reqwest/async_impl/request/struct.RequestBuilder.html\" title=\"struct reqwest::async_impl::request::RequestBuilder\">ReqwestRequestBuilder</a>&gt;</h4></section><section id=\"method.body_ref\" class=\"method\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#110-112\">source</a><h4 class=\"code-header\">pub fn <a href=\"serenity/http/struct.Request.html#tymethod.body_ref\" class=\"fn\">body_ref</a>(&amp;self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;&amp;[<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u8.html\">u8</a>]&gt;</h4></section><section id=\"method.body_mut\" class=\"method\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#115-117\">source</a><h4 class=\"code-header\">pub fn <a href=\"serenity/http/struct.Request.html#tymethod.body_mut\" class=\"fn\">body_mut</a>(&amp;mut self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;&amp;mut [<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u8.html\">u8</a>]&gt;</h4></section><section id=\"method.headers_ref\" class=\"method\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#120-122\">source</a><h4 class=\"code-header\">pub fn <a href=\"serenity/http/struct.Request.html#tymethod.headers_ref\" class=\"fn\">headers_ref</a>(&amp;self) -&gt; &amp;<a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"https://docs.rs/http/0.2.11/http/header/map/struct.HeaderMap.html\" title=\"struct http::header::map::HeaderMap\">Headers</a>&gt;</h4></section><section id=\"method.headers_mut\" class=\"method\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#125-127\">source</a><h4 class=\"code-header\">pub fn <a href=\"serenity/http/struct.Request.html#tymethod.headers_mut\" class=\"fn\">headers_mut</a>(&amp;mut self) -&gt; &amp;mut <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"struct\" href=\"https://docs.rs/http/0.2.11/http/header/map/struct.HeaderMap.html\" title=\"struct http::header::map::HeaderMap\">Headers</a>&gt;</h4></section><section id=\"method.method_ref\" class=\"method\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#130-132\">source</a><h4 class=\"code-header\">pub fn <a href=\"serenity/http/struct.Request.html#tymethod.method_ref\" class=\"fn\">method_ref</a>(&amp;self) -&gt; &amp;<a class=\"enum\" href=\"serenity/http/enum.LightMethod.html\" title=\"enum serenity::http::LightMethod\">LightMethod</a></h4></section><section id=\"method.route_ref\" class=\"method\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#135-137\">source</a><h4 class=\"code-header\">pub fn <a href=\"serenity/http/struct.Request.html#tymethod.route_ref\" class=\"fn\">route_ref</a>(&amp;self) -&gt; &amp;<a class=\"enum\" href=\"serenity/http/enum.Route.html\" title=\"enum serenity::http::Route\">Route</a>&lt;'_&gt;</h4></section><section id=\"method.params_ref\" class=\"method\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#140-142\">source</a><h4 class=\"code-header\">pub fn <a href=\"serenity/http/struct.Request.html#tymethod.params_ref\" class=\"fn\">params_ref</a>(&amp;self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;&amp;[(&amp;'static <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/string/struct.String.html\" title=\"struct alloc::string::String\">String</a>)]&gt;</h4></section><section id=\"method.params_mut\" class=\"method\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#145-147\">source</a><h4 class=\"code-header\">pub fn <a href=\"serenity/http/struct.Request.html#tymethod.params_mut\" class=\"fn\">params_mut</a>(&amp;mut self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;&amp;mut [(&amp;'static <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.str.html\">str</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/alloc/string/struct.String.html\" title=\"struct alloc::string::String\">String</a>)]&gt;</h4></section></div></details>",0,"serenity::http::request::RequestBuilder"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-Request%3C'a%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#23\">source</a><a href=\"#impl-Debug-for-Request%3C'a%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"serenity/http/struct.Request.html\" title=\"struct serenity::http::Request\">Request</a>&lt;'a&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#23\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/nightly/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","serenity::http::request::RequestBuilder"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Clone-for-Request%3C'a%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#23\">source</a><a href=\"#impl-Clone-for-Request%3C'a%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"serenity/http/struct.Request.html\" title=\"struct serenity::http::Request\">Request</a>&lt;'a&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/serenity/http/request.rs.html#23\">source</a><a href=\"#method.clone\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone\" class=\"fn\">clone</a>(&amp;self) -&gt; <a class=\"struct\" href=\"serenity/http/struct.Request.html\" title=\"struct serenity::http::Request\">Request</a>&lt;'a&gt;</h4></section></summary><div class='docblock'>Returns a copy of the value. <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone_from\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/clone.rs.html#169\">source</a></span><a href=\"#method.clone_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from\" class=\"fn\">clone_from</a>(&amp;mut self, source: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;Self</a>)</h4></section></summary><div class='docblock'>Performs copy-assignment from <code>source</code>. <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from\">Read more</a></div></details></div></details>","Clone","serenity::http::request::RequestBuilder"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()