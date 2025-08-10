// Central component registry
// Imports all component factory functions and registers them with the shared Base via $.define

import AppSnippet from '../../components/app-snippet.js'
import AppButton from '../../components/button.js'
import Hooks from '../../components/hooks.js'
import { $, App } from '../../components/index.js'
import Nav from '../../components/navbar.js'
import AppSegmented from '../../components/segmented.js'
import Sidebar from '../../components/sidebar.js'
import Snippet from '../../components/snippet.js'
import Tabs from '../../components/tabs.js'
import Txt from '../../components/txt.js'

const registry = [
  ['cd-nav', Nav],
  ['cd-tabs', Tabs],
  ['cd-snippet', Snippet],
  ['cd-hooks', Hooks],
  ['cd-sidebar', Sidebar],
  ['cd-txt', Txt],
]

registry.forEach(([tag, fn]) => $.define(tag, fn))

// Register new light-DOM primitives
App.define('app-text', Txt)
App.define('app-btn', AppButton)
App.define('app-segmented', AppSegmented)
App.define('app-snippet', AppSnippet)


