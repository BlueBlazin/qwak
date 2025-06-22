# Qwk

A command line tool for creating command line aliases for AI agents to perform on pre-defined tasks.

**A concrete example:**

I often find myself initializing a new React project with vite. For that I usually go through the following steps:

Run `bun create vite . --template react-swc`
Delete `src/assets/react.svg`, `public/vite.svg`, `App.css`
Run `bun add styled-components`.
Replace `src/index.css` with a very simple css reset containing just:

```css
:root {
  font-family: system-ui, Avenir, Helvetica, Arial, sans-serif;
  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}
```

Replace `src/App.jsx` with a simple component that creates a styled-components `Container` styled div and returns just `<Contianer>App</Container>`.
Have `Container` in `src/App.jsx` take up 100vw and 100vh.

Instead of manually doing all the above steps I just want to write this prompt once and have claude code or another agent do it all for me from within the empty project directory.

I want my tool to be able to create aliases for running such predefined agentic prompts, such as:

```sh
qwk vite-react-swc
```

to run my choice of agent with the above prompt.

To set a prompt you should be able to do:

```sh
qwk --set vite-react-swc "example prompt"
```

or

```sh
cat prompt.txt | qwk --set vite-react-swc
```

or just

```sh
qwk --set vite-react-swc
>
```

and then paste the prompt.

To set the agent trigger command:

```sh
qwk --agent claude
```
