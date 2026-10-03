use topcoat::{context::Cx, view::view};
#[cfg(target_arch = "wasm32")]
mod edge;
#[cfg(target_arch = "wasm32")]
mod email;

/// The same Topcoat render is used by the Worker and the static export.
pub async fn render() -> topcoat::Result<String> {
    let cx = Cx::default();
    let document = landing(&cx).await?;
    Ok(document.render(&cx))
}

async fn landing(cx: &Cx) -> topcoat::Result {
    view! {
        cx =>
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <meta name="description" content="Relay is an open-source desktop app for AI chat, tasks, and coding. Use your own API keys and server. Join the waitlist for launch updates.">
                <meta name="theme-color" content="#f3f5f8">
                <title>"Relay — One desktop app for AI chat, tasks, and code."</title>
                <link rel="icon" href="/favicon.svg" type="image/svg+xml">
                <link rel="stylesheet" href="/style.css">
                <script src="/client/boot.js" type="module"></script>
            </head>
            <body>
                <a class="skip-link" href="#main">"Skip to content"</a>
                <div class="page-shell">
                    <header class="site-header">
                        <a class="brand" href="#" aria-label="Relay home"><img class="brand-mark" src="/relay-mark.svg" alt="" width="36" height="36">"relay"</a>
                        <nav aria-label="Main navigation">
                            <a href="#workspace">"Preview"</a>
                            <a href="#principles">"Features"</a>
                            <a class="github-link" href="https://github.com/MrSheerluck/smbl-relay" target="_blank" rel="noopener noreferrer" aria-label="Relay source code on GitHub" title="View Relay on GitHub"><img src="/github-mark.svg" alt="" width="18" height="18"><span>"GitHub"</span></a>
                            <a class="nav-cta" href="#waitlist">"Join waitlist"</a>
                        </nav>
                    </header>
                    <main id="main">
                        <section class="hero" aria-labelledby="hero-title">
                            <div class="hero-copy">
                                <p class="eyebrow"><span class="status-dot"></span>"A DESKTOP AI APP · IN DEVELOPMENT"</p>
                                <h1 id="hero-title">"AI chat, tasks,"<br><span>"and code."</span></h1>
                                <p class="hero-description">"Relay is a desktop app for chatting with AI, getting tasks done, and working on code. Use models from OpenAI, Anthropic, Google, and more with your own API keys and server."</p>
                                <form class="waitlist-form" id="waitlist" action="/api/waitlist" method="post" aria-labelledby="waitlist-label">
                                    <label id="waitlist-label" for="waitlist-email">"Get an email when Relay is ready."</label>
                                    <div class="waitlist-fields"><input id="waitlist-email" name="email" type="email" autocomplete="email" inputmode="email" placeholder="Your email address" maxlength="254" required="" aria-describedby="waitlist-note waitlist-status"><button class="button primary" type="submit">"Join waitlist"</button></div>
                                    <p class="waitlist-note" id="waitlist-note">"We’ll use your email for Relay launch updates."</p>
                                    <p class="waitlist-status" id="waitlist-status" role="status" aria-live="polite" aria-atomic="true"></p>
                                </form>
                                <div class="hero-actions">
                                    <a class="text-link" href="#workspace">"See the app preview "<span aria-hidden="true">"↓"</span></a>
                                    <a class="text-link" href="#roadmap">"View the release plan "<span aria-hidden="true">"→"</span></a>
                                </div>
                                <p class="hero-note">"Designed for macOS, Windows & Linux."</p>
                            </div>
                            <div class="signal-art" aria-hidden="true">
                                <div class="signal-ring ring-outer"></div><div class="signal-ring ring-middle"></div><div class="signal-ring ring-inner"></div>
                                <span class="signal-label label-in">"YOUR MODELS"</span><span class="signal-label label-out">"ONE APP"</span>
                                <div class="signal-core"><img src="/relay-mark.svg" alt="" width="54" height="54"></div><span class="orbit-node node-one"></span><span class="orbit-node node-two"></span><span class="orbit-node node-three"></span>
                            </div>
                        </section>
                        <section class="workspace-section" id="workspace" aria-labelledby="workspace-title">
                            <div class="section-kicker"><h2 id="workspace-title">"Chat with AI. Get tasks done. Write code."</h2><span>"APP DESIGN PREVIEW"</span></div>
                            <div class="app-window">
                                <div class="window-bar"><div class="traffic-lights" aria-hidden="true"><i></i><i></i><i></i></div><span>"Relay / Personal workspace"</span><span class="window-shortcut">"⌘ K"</span></div>
                                <div class="app-body">
                                    <aside class="app-sidebar" aria-label="Preview workspace">
                                        <div class="sidebar-brand"><img class="mini-mark" src="/relay-mark.svg" alt="" width="23" height="23">"My workspace"</div>
                                        <div class="mode-tabs" role="tablist" aria-label="Preview mode">
                                            <button id="tab-chat" class="mode-tab active" role="tab" aria-selected="true" aria-controls="preview-panel" data-mode="chat">"◌ "<span>"Chat"</span><kbd>"1"</kbd></button>
                                            <button id="tab-work" class="mode-tab" role="tab" aria-selected="false" aria-controls="preview-panel" tabindex="-1" data-mode="work">"▧ "<span>"Work"</span><kbd>"2"</kbd></button>
                                            <button id="tab-code" class="mode-tab" role="tab" aria-selected="false" aria-controls="preview-panel" tabindex="-1" data-mode="code">"⌘ "<span>"Code"</span><kbd>"3"</kbd></button>
                                        </div>
                                        <p class="sidebar-label">"RECENT CHATS"</p>
                                        <div class="recent-chats"><span>"A better morning routine"</span><span>"Ideas for the next release"</span><span>"Understanding async Rust"</span></div>
                                        <div class="sidebar-bottom"><span class="avatar">"Y"</span><div>"Your workspace"<small>"Your own infrastructure"</small></div></div>
                                    </aside>
                                    <div class="app-main" id="preview-panel" role="tabpanel" aria-labelledby="tab-chat" tabindex="0">
                                        <div class="preview-topline"><span id="mode-caption">"Chat with AI."</span><span class="local-indicator">"App preview"</span></div>
                                        <div class="preview-content">
                                            <span class="preview-symbol" id="preview-symbol" aria-hidden="true">"✳"</span>
                                            <h3 id="preview-title">"What would you like to ask?"</h3>
                                            <p id="preview-description">"Ask a question, explore an idea, or learn something new."</p>
                                            <div class="prompt-chips" id="prompt-chips"><button type="button">"Think through an idea"</button><button type="button">"Explain something"</button><button type="button">"Explore a document"</button></div>
                                        </div>
                                        <form class="composer" id="preview-form">
                                            <label class="sr-only" for="preview-input">"Try a sample prompt"</label>
                                            <textarea id="preview-input" rows="2" maxlength="500" placeholder="What’s on your mind?"></textarea>
                                            <div class="composer-controls"><span class="composer-hint">"Sample interaction · no AI connected"</span><button class="send-button" type="submit" aria-label="Try this prompt">"↑"</button></div>
                                        </form>
                                        <p class="preview-feedback" id="preview-feedback" role="status" aria-live="polite"></p>
                                    </div>
                                </div>
                            </div>
                            <p class="preview-footnote">"This is a design preview. Relay is still in development, and no AI is connected here."</p>
                        </section>
                        <section class="principles" id="principles" aria-labelledby="principles-title">
                            <div class="principles-intro"><p class="eyebrow">"PLANNED FEATURES"</p><h2 id="principles-title">"Your AI. "<br>"Your choice."</h2><p>"Choose the models you use and where your data lives. Relay won’t require a Relay account."</p></div>
                            <div class="feature-grid">
                                for (icon, title, description) in [
                                    ("⇄", "Choose your AI models", "Use models from OpenAI, Anthropic, Google, and other providers with your own API keys."),
                                    ("⌁", "A desktop app", "Built for macOS, Windows, and Linux, with keyboard shortcuts and saved conversations."),
                                    ("▣", "Run your own server", "Store your conversations on a server you control. Connect your desktop app to it."),
                                    ("↗", "Open source", "Read the source code, suggest changes, or help build Relay. The app’s code will be open to everyone."),
                                ] {
                                    <article class="feature"><span class="feature-icon" aria-hidden="true">(icon)</span><h3>(title)</h3><p>(description)</p></article>
                                }
                            </div>
                        </section>
                        <section class="roadmap" id="roadmap" aria-labelledby="roadmap-title">
                            <div><p class="eyebrow">"RELEASE PLAN"</p><h2 id="roadmap-title">"Chat comes first."</h2><p>"The first release will let you chat with AI, save conversations, and switch models. Tools for longer tasks and coding will come later."</p></div>
                            <ol class="roadmap-steps"><li><span class="step-marker">"01"</span><div><h3>"Chat"</h3><p>"Ask questions and save conversations"</p></div><span class="step-state">"First release"</span></li><li><span class="step-marker">"02"</span><div><h3>"Work"</h3><p>"Research topics, make files, and review results"</p></div><span class="step-state">"Later"</span></li><li><span class="step-marker">"03"</span><div><h3>"Code"</h3><p>"Read, edit, and test code in your projects"</p></div><span class="step-state">"Later"</span></li></ol>
                        </section>
                    </main>
                    <footer><a class="brand" href="#"><img class="brand-mark" src="/relay-mark.svg" alt="" width="28" height="28">"relay"</a><p>"One desktop app for AI chat, tasks, and code."</p><span>"OPEN SOURCE · IN DEVELOPMENT"</span></footer>
                </div>
            </body>
        </html>
    }
}
