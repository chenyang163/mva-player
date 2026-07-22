# MVA Player Agent Rules


# Project Overview


## Project Name

MVA Player


## Meaning

Music Visual Animation Player


## Project Vision


This project explores a timeline-based music visual runtime.


The goal is to combine:


- Audio
- Lyrics
- Timeline data
- Dynamic typography animation
- Visual effects
- User customization
- Plugin extensions


The project aims to provide a new way to describe and render music visual experiences.


The long-term direction is similar to:


- VLC (media playback)
- Spotify / NetEase Cloud style lyrics experience
- After Effects timeline-based composition


However:

This project is an experimental open-source project.

Do not assume it is already a standard or ecosystem.


--------------------------------------------------


# Core Development Rules


## Rule 1: Research Before Major Implementation


Before implementing any major feature or introducing a dependency:


Investigate existing solutions first.


Search:

- GitHub
- crates.io
- Official documentation
- Existing open source projects


Priority:


1. Mature open source solution

2. Existing Rust libraries

3. Existing projects that can be adapted

4. Self implementation


Do NOT recreate mature technology without reason.


Dependency decisions must be documented in:


docs/dependencies.md


Include:


- Project name
- Repository URL
- License
- Purpose
- Reason for choosing



Small changes such as:

- bug fixes
- formatting
- documentation updates
- simple tests

do not require external research.


--------------------------------------------------


# Technology Requirements


## Backend Language


Must use:


Rust


Requirements:


- Modular architecture
- Strong error handling
- Maintainable code
- Async support when necessary


Do not use another language for core functionality.


--------------------------------------------------


# UI Framework


Before selecting or changing UI framework:


Research Rust UI ecosystem.


Candidates:


- Slint
- Dioxus Desktop
- Iced
- egui


Evaluate:


- Windows support
- Cross-platform capability
- Performance
- Community activity
- Long-term maintenance


Record decisions in documentation.


--------------------------------------------------


# Project Architecture


The project must be modular.


However:


Do not create empty modules only for future ideas.


A module should exist when:


- It has clear responsibility
- It contains real implementation
- Separation improves maintainability


Recommended architecture direction:


mva-player


├── core

├── audio

├── decoder

├── lyrics

├── renderer

├── format

├── editor

├── plugin

├── ui

├── settings

└── main.rs



Module responsibility:


core:

Application state and communication.


audio:

Audio playback engine.


decoder:

Audio decoding.


lyrics:

Lyrics and subtitle processing.


renderer:

Animation and visual rendering.


format:

MVA format handling.


editor:

MVA creation and editing tools.


plugin:

Extension system.


ui:

User interface.


settings:

User configuration.


--------------------------------------------------


# Configuration First Rule


Before implementing configurable features:


Step 1:

Create configuration items.


Example:


config/audio.toml


Contains:


volume

buffer_size

decoder_threads



Step 2:

Create Rust configuration structures.


Example:


AudioConfig



Step 3:

Business code reads configuration.


Never hardcode user-adjustable values.


--------------------------------------------------


# Configuration Directory


Adjustable parameters should exist in config folder.


Example:


config/


├── app.toml

├── audio.toml

├── renderer.toml

├── plugin.toml

├── ui.toml

└── editor.toml



--------------------------------------------------


# MVA Format Design


Target format:


.mva


Concept:


MP3:

Audio only


MP4:

Audio + Video


MVA:

Audio + Visual Experience



Possible structure:


song.mva


audio/

lyrics/

animation/

metadata/


Future support:


- Audio
- Lyrics
- Animation
- Images
- Shaders
- Plugins


Before format stabilization:


Breaking changes are acceptable.


Design with future compatibility in mind.



--------------------------------------------------


# Initial Supported Formats


First version:


Audio:


- MP3
- FLAC
- WAV


Lyrics:


- LRC
- ASS



First milestone:


Music playback

+

Synchronized lyrics



--------------------------------------------------


# Editor System


The application may eventually include:


MVA Creator


Functions:


Import:


Audio file

+

Lyrics file



Edit:


- Lyrics timing
- Font
- Animation
- Effects


Export:


.mva



--------------------------------------------------


# Animation System


Animation must be data-driven.


Do not hardcode animations.


Example:


animation.json


{
"time":10,
"text":"Hello",
"animation":"scale"
}



Support:


- Position
- Scale
- Rotation
- Opacity
- Particle effects
- Shader effects


--------------------------------------------------


# Plugin System


The application should support future extensions.


Possible plugins:


- Audio formats
- Lyrics formats
- Animation effects
- Themes
- Tools


Plugin architecture should be designed when real requirements appear.


Do not implement complex plugin infrastructure without use cases.


--------------------------------------------------


# Development Workflow


Every major feature follows:


1. Research existing solutions

2. Design architecture

3. Create configuration

4. Implement code

5. Test

6. Update documentation

7. Create demo/example if appropriate



--------------------------------------------------


# Documentation


Maintain:


docs/


architecture.md

format.md

plugin.md

config.md

dependencies.md

roadmap.md

specification.md



Documentation must stay synchronized with implementation.


--------------------------------------------------


# Code Quality


Always run:


cargo fmt

cargo clippy


Avoid:


- Excessive unwrap()
- Hardcoded values
- Large unstructured files
- Unnecessary dependencies



--------------------------------------------------


# AI Generated Code Review


AI generated code is not trusted by default.


Before accepting generated code:


Check:


- License compatibility
- Similarity with existing projects
- Security issues
- Error handling
- Performance impact


Never copy external code without verifying license compatibility.


AI is an assistant, not the author of the project.



--------------------------------------------------


# Git Workflow


## Commit Rules


Every commit should represent one logical change.


Commit format:


<type>: <description>



Examples:


feat: add lyric animation timeline


fix: resolve audio initialization failure


docs: update MVA specification


refactor: simplify renderer pipeline


ci: add linux audio dependencies



Avoid:


- update
- modify
- changes
- fix stuff
- temporary


Do not mix unrelated changes in one commit.



--------------------------------------------------


# Remote Repository Rules


Agents MUST NOT:


- Create remote repositories
- Push automatically
- Force push
- Rewrite history


Before pushing:


Ask for confirmation.


Allowed:


- Prepare commits
- Show git status
- Provide push commands



--------------------------------------------------


# Release Rules


Versioning:


v0.x:

Experimental development.


v1.0:

Stable public API.


Before release:


Update:


- README.md
- CHANGELOG.md
- Documentation


Create Git tag.



--------------------------------------------------


# Demo Requirements


Major milestones should include a demonstrable example.


A demo should show:


- Why the feature exists
- User experience
- Core project value


Prefer:


Visual demonstrations

over:


Only technical benchmarks.



--------------------------------------------------


# Agent Behavior


Do not generate the whole project at once.


Work incrementally.


After each milestone explain:


1. What was implemented

2. Why this design was chosen

3. Which libraries were used

4. What files changed

5. Next development step


Do not expand project scope without discussion.

