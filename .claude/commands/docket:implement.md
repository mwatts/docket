# Docket Implementation Guide

You are helping implement a bug tracked by docket. Read the current bug context and guide the implementation work.

## Instructions

1. **Read the bug context** from `.docket/current.json` to understand what needs to be implemented. Parse the JSON to extract:
   - `title`: The bug title
   - `body`: Contains the Goal, Acceptance Criteria, Context, and Log sections
   - `bug_id`: The bug identifier
   - `priority`: The bug priority
   - `status`: Current status

2. **Understand the requirements** by carefully reading:
   - The **Goal** section to understand what needs to be accomplished
   - The **Acceptance Criteria** to know exactly what must be delivered
   - The **Context** section for background information and constraints

3. **Plan and implement** the work:
   - Create a todo list to track progress through the acceptance criteria
   - Work through each acceptance criterion systematically
   - Follow best practices for the codebase

4. **Update progress** as you work:
   - After completing significant milestones, update the bug's Log section in `.docket/current.json`
   - Add timestamped entries describing what was accomplished
   - Format log entries as: `- YYYY-MM-DD: Description of progress`

5. **Check off acceptance criteria** as they are completed:
   - Update `.docket/current.json` to change `- [ ]` to `- [x]` for completed criteria
   - Only mark criteria as complete when fully verified

## Getting Started

Begin by reading `.docket/current.json` and presenting a summary of the bug to the user, then propose an implementation plan.
