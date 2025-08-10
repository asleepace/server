# About

This document outlines a proposal for creating a custom file type that will be used to generate client-side JS, HTML and CSS. The main goals of this file are as follows:

1. Create interactive, reusable, client components in plan HTML, CSS and JS.
2. Transpiled at run-time to avoid build steps and external deps.
3. Can use normal .CSS, .HTML and .JS syntax highlighting in one file
4. Similar to (.astro) files or (.vue) files, should interop with VSCode / Cursor
5. Configurable and composable

The current plan is to have these custom `.wc` files get converted to custom web components.

## Basic Syntax

I would like the basic syntax to feel similar to Objective-C where the entire file itself is treated as a class, this will then make it easier to generate the custom (.js) web components.

```.wc
@component user-profile: HTMLElement
@prop (attr) userName: String
@prop (attr) userId: Number
@prop (attr) avatarUrl: String

@style
  :host { display:flex; gap:.5rem; align-items:center; }
  img { width:40px; height:40px; border-radius:999px; }
  .name { font-weight:600; }
@end

@template
  <img src="{{avatarUrl}}" alt="" />
  <span class="name">{{userName}}</span>
@end

@method onMounted: () => {
  // typed accessors for (attr) props:
  // this.userId -> number, this.userName -> string
}
```

This is just a rough idea and can be completely changed, the main thing to notice is the following:

```
// imports
@import 'file...'

// properties
@property (modifiers) name: ...args

// langauge blocks
@block (lang) optionalName: ...args
  // lang specific syntax here
@end

// methods
@method methodName(...args) {}
```

The goal being to make it easier to parse and define.

## Specification (MVP)

- File: .wc
- Directives (must start at line-begin with @, allow leading spaces):
  - @component tag-name: BaseClass
  - @prop (modifiers?) name: Type (= default?)
    - Supported Types: String | Number | Boolean (mapping to JS constructors)
    - Modifier (attr): binds attribute as data-attr-name and generates a typed getter: this.name
  - @style … @end: CSS content (embedded CSS syntax highlighting)
  - @template … @end: HTML content with:
    - Text/attr bindings: {{prop}}
    - Events: on:click="methodName"
  - @method name: () => { … }: JS method body (embedded JS)
  - @script … @end: arbitrary JS block (embedded JS)

### Runtime mapping

- Compiles to a light-DOM component using App.define(tag, renderFn)
- (attr) props become getters reading data-attr-\* with coercion via constructor:
  - String: String(value)
  - Number: Number(value)
  - Boolean: value === 'true' or empty string
- JSDoc emitted:
  - @typedef for component props
  - @returns on generated getters for intellisense
- Template transforms:
  - {{prop}} → interpolation
  - on:click="save" → delegated listener calls this.save(e)

### Example usage

```html
<link rel="wc" href="/client/wc/app-text.wc" />
<app-text data-attr-text="Hello" data-attr-variant="h2"></app-text>
```

### Notes

- Reflect is the default behavior; omit unless needed later
- Keep expressions minimal (identifiers only) for predictability
