A Rust-based, in-memory event ticketing system with support for event creation, ticket purchase, resale, user roles, organizers, and basic NFT-like metadata (DIP-721). This is a modular back-end logic demo: it is not a complete web/mobile service, but serves as a solid foundation for building a decentralized or centralized ticketing platform.

Features
Event Management

Create, update, view, cancel events (with authorization)

Ticket Purchase & Resale

Purchase primary tickets from the organizer

Allow resale of tickets with price caps (anti-scalping)

Transfer tickets between users

Validate/invalidate tickets (for entry, anti-fraud, etc.)

Role-Based Access Control

Admin, Organizer, Regular User roles

Admins can assign roles, organizers can manage their events

NFT & Metadata Support

Each ticket has unique metadata and tracks its history (mimics DIP-721 interface)

Statistics

Access ticket sales and revenue stats per event

Data Model Overview
Principal: Represents a user (wrapper for a string ID)

Role: {Admin, Organizer, User}

Event: Contains details about the event

Ticket: Uniquely identifies an entry right, includes metadata, owner, validity, price history

TicketTransfer: Tracks ticket transfers or sales

TicketMetadata/TokenMetadata: Extra data per ticket, simulates NFT metadata

TicketingSystem: Main struct, holds all storage (events, tickets, users, mapping...)
