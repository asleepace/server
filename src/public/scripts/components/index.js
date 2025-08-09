// Central component registry
// Imports all component factory functions and registers them with the shared Base via $.define

import { $ } from '../../components/index.js'
import Nav from '../../components/navbar.js'
import Snippet from '../../components/snippet.js'
import Tabs from '../../components/tabs.js'

const registry = [
    ['cd-nav', Nav],
    ['cd-tabs', Tabs],
    ['cd-snippet', Snippet],
]

registry.forEach(([tag, fn]) => $.define(tag, fn))


