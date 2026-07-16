## Progress Marker
First commit contains only the rough sketch of the template for the chat app. 

I logged the several changes that I made in toml specifically so as to track the various ways in which functionality between web frameworks for rust differs. 

I have tested functionality/preferences in building the code, between **Rocket** (I used it first because of its beginner-friendliness) and using **Tokio** directly; main.rs implements traits using Rocket while the files in /bin use Tokio directly. This decision in file arrangement is only for the purpose of compactness, neatness and avoiding confusion.

Second commit includes neater files with error handling and stream connection handling

The front-end is curerntly unpolished and not personalized to my specific aesthetic preferences, and some parts of it may have been commented out to avoid unecessary complexitieties while compiling. The front-end will be polished once I have learned/mastered the back-end and it is fully functional as per the vision of the project.

Furthermore, I have come across Axum framework which suits my needs for API development, therefore the latest version of toml contains dependencies that support Axum. 

This has as well made me curious about other tools for web development with rust: Actix, Tide etc. For now, I am extensively learning with Axum as my preferred tool.

/bin directory contains the workflow for Axum.




