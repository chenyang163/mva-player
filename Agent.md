# MVA Player Agent Rules


## Project Overview

Project Name:

MVA Player

Meaning:

Music Visual Animation Player


This project is creating a next-generation music player.

The goal is:

A music file should not only contain audio.

It should support:

- Audio
- Lyrics
- Timeline data
- Dynamic typography animation
- Visual effects
- Plugin extensions
- User customization


The final product should be similar to:

VLC + Spotify/NetEase Cloud style lyrics + After Effects timeline.


--------------------------------------------------


# Core Development Rules


## Rule 1: Research Before Implementation


Before implementing any feature:

You MUST investigate existing solutions first.


Search:

- GitHub
- crates.io
- Official documentation
- Existing open source projects


Priority order:

1. Mature open source solution

2. Existing Rust libraries

3. Existing projects that can be modified

4. Self implementation


Do NOT recreate existing mature technology.


Every third-party dependency decision must be documented in:

docs/dependencies.md


Include:

- Project name
- Repository URL
- License
- Purpose
- Reason for choosing


--------------------------------------------------


# Technology Requirements


## Backend Language


Must use:

Rust


Requirements:

- Modular architecture
- Strong error handling
- Async support when necessary
- Maintainable code


Do not use another language for core functions.


--------------------------------------------------


# UI Framework


Before choosing UI framework:

Research current Rust UI ecosystem.


Candidates:

- Slint
- Dioxus Desktop
- Iced
- egui


If Microsoft's Rust UI ecosystem has a mature solution:

Evaluate it.


Selection criteria:

- Windows support
- Cross-platform capability
- Performance
- Community activity
- Long-term maintenance


The decision must be recorded.


--------------------------------------------------


# Project Architecture


The project must be modular.


Recommended structure:


mva-player

├── config

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

├── about

└── main.rs



Module responsibility:


core:

Application state and communication.


audio:

Audio playback engine.


decoder:

Audio format decoding.


lyrics:

Subtitle and lyrics parsing.


renderer:

Animation rendering engine.


format:

MVA format handling.


editor:

Music visual editing.


plugin:

Plugin system.


ui:

User interface.


settings:

Application settings.


about:

About page.


--------------------------------------------------


# Configuration First Rule


IMPORTANT:


Before writing any feature code:


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


Never hardcode user adjustable values.


--------------------------------------------------


# Configuration Directory


All adjustable parameters must exist in config folder.


Example:


config

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


Maintain backward compatibility.


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


The application should eventually include:

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


Animation must be data driven.


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


The application must support plugins.


Plugins can provide:


- New audio formats
- New lyric formats
- New animation effects
- New themes
- New tools


Plugin directory:


plugins/



Design for dynamic loading.


--------------------------------------------------


# Development Workflow


Every feature follows this process:


1. Research existing solutions

2. Design architecture

3. Create configuration

4. Implement code

5. Test

6. Update documentation



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



--------------------------------------------------


# Code Quality


Always run:


cargo fmt

cargo clippy



Avoid:

- Excessive unwrap()
- Hardcoded values
- Large unstructured files


--------------------------------------------------


# Agent Behavior


Do not generate the whole project at once.


Work incrementally.


After each milestone explain:


1. What was implemented

2. Why this design was chosen

3. Which libraries were used

4. Next development step

