# Internationalization Guide

> **⚠️ Development Status**: Localization support is in the early stages. Expect limited language coverage.

This guide explains how to enable multiple languages in ICN Core applications and how we approach accessibility.

## Setup

Localization files live under `apps/*/locales/`. Add new language JSON files following the existing format. Update the `i18n` configuration in each frontend app to register the new locale.

## Accessibility

* Ensure UI components have appropriate ARIA labels.
* Test color contrast and keyboard navigation.
* Provide alternative text for all images.

Community contributions are welcome to expand language support and improve accessibility practices.
