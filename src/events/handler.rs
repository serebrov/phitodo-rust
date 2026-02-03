use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::{App, AppMode, FocusArea};
use crate::models::TaskStatus;
use crate::ui::theme::SidebarItem;

/// Handle a key event and return whether to continue running
pub fn handle_key_event(app: &mut App, key: KeyEvent) -> bool {
    // Global shortcuts that work in any mode
    // Allow quit with 'q' unless in input mode, task form, or actively editing in settings
    let in_settings_editing = app.mode == AppMode::Settings && app.settings_view.editing;
    if key.code == KeyCode::Char('q') && !matches!(app.mode, AppMode::Input | AppMode::TaskForm) && !in_settings_editing {
        return false;
    }

    if key.code == KeyCode::Char('?') && app.mode == AppMode::Normal {
        app.show_help = !app.show_help;
        return true;
    }

    if app.show_help {
        // Any key closes help
        app.show_help = false;
        return true;
    }

    // Handle mode-specific input
    match app.mode {
        AppMode::Normal => handle_normal_mode(app, key),
        AppMode::Input => handle_input_mode(app, key),
        AppMode::TaskForm => handle_task_form_mode(app, key),
        AppMode::Confirm => handle_confirm_mode(app, key),
        AppMode::Settings => handle_settings_mode(app, key),
    }

    true
}

fn handle_normal_mode(app: &mut App, key: KeyEvent) {
    // Alt+number shortcuts for priority
    if key.modifiers.contains(KeyModifiers::ALT) {
        if let KeyCode::Char(c) = key.code {
            match c {
                '1' => app.set_task_priority(crate::models::TaskPriority::None),
                '2' => app.set_task_priority(crate::models::TaskPriority::Low),
                '3' => app.set_task_priority(crate::models::TaskPriority::Medium),
                '4' => app.set_task_priority(crate::models::TaskPriority::High),
                _ => {}
            }
        }
        return;
    }

    match key.code {
        // View switching with number keys
        KeyCode::Char('1') => app.switch_to_view(SidebarItem::Inbox),
        KeyCode::Char('2') => app.switch_to_view(SidebarItem::Today),
        KeyCode::Char('3') => app.switch_to_view(SidebarItem::Upcoming),
        KeyCode::Char('4') => app.switch_to_view(SidebarItem::Anytime),
        KeyCode::Char('5') => app.switch_to_view(SidebarItem::Completed),
        KeyCode::Char('6') => app.switch_to_view(SidebarItem::Review),
        KeyCode::Char('7') => app.switch_to_view(SidebarItem::GitHub),
        KeyCode::Char('8') => app.switch_to_view(SidebarItem::Toggl),
        KeyCode::Char('9') => app.switch_to_view(SidebarItem::Settings),

        // Navigation
        KeyCode::Tab => app.cycle_focus(),
        KeyCode::BackTab => app.cycle_focus_reverse(),

        KeyCode::Char('j') | KeyCode::Down => app.select_next(),
        KeyCode::Char('k') | KeyCode::Up => app.select_previous(),
        KeyCode::Char('g') => app.select_first(),
        KeyCode::Char('G') => app.select_last(),
        KeyCode::Char('h') | KeyCode::Left => {
            if app.focus == FocusArea::Detail {
                app.focus = FocusArea::List;
            } else if app.focus == FocusArea::List {
                app.focus = FocusArea::Sidebar;
            }
        }
        KeyCode::Char('l') | KeyCode::Right => {
            if app.focus == FocusArea::Sidebar {
                app.focus = FocusArea::List;
            } else if app.focus == FocusArea::List {
                app.focus = FocusArea::Detail;
            }
        }

        // Task actions
        KeyCode::Char(' ') => app.toggle_task_completed(),
        KeyCode::Char('n') => app.start_new_task(),
        KeyCode::Char('N') => app.start_new_project(),
        KeyCode::Char('e') => app.start_edit_task(),
        KeyCode::Char('d') => app.start_delete(),
        KeyCode::Char('o') => app.open_task_url(),

        // Status shortcuts
        KeyCode::Char('i') => app.set_task_status(TaskStatus::Inbox),
        KeyCode::Char('a') => app.set_task_status(TaskStatus::Active),
        KeyCode::Char('s') => app.set_task_status(TaskStatus::Scheduled),

        // Search
        KeyCode::Char('/') => app.start_search(),

        // Refresh
        KeyCode::Char('r') => app.refresh_data(),

        // Enter on sidebar or list
        KeyCode::Enter => app.activate_selected(),

        _ => {}
    }
}

fn handle_input_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.cancel_input(),
        KeyCode::Enter => app.submit_input(),
        KeyCode::Backspace => app.input.backspace(),
        KeyCode::Delete => app.input.delete(),
        KeyCode::Left => app.input.move_left(),
        KeyCode::Right => app.input.move_right(),
        KeyCode::Home => app.input.move_start(),
        KeyCode::End => app.input.move_end(),
        KeyCode::Char(c) => app.input.insert(c),
        _ => {}
    }
}

fn handle_task_form_mode(app: &mut App, key: KeyEvent) {
    let Some(ref mut form) = app.task_form else {
        app.mode = AppMode::Normal;
        return;
    };

    match key.code {
        KeyCode::Esc => {
            app.task_form = None;
            app.mode = AppMode::Normal;
        }
        KeyCode::Enter => {
            // If on a text field, could be submitting. Otherwise save the form.
            use crate::ui::components::TaskFormField;
            match form.current_field {
                TaskFormField::Title | TaskFormField::Notes | TaskFormField::DueDate => {
                    // Check if shift is held for submit
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        form.apply_inputs();
                        app.save_task_form();
                    }
                }
                _ => {
                    form.apply_inputs();
                    app.save_task_form();
                }
            }
        }
        KeyCode::Tab => form.next_field(),
        KeyCode::BackTab => form.prev_field(),

        // Handle cycling for select fields
        KeyCode::Left | KeyCode::Right => {
            use crate::ui::components::TaskFormField;
            match form.current_field {
                TaskFormField::Project => form.cycle_project(),
                TaskFormField::Priority => form.cycle_priority(),
                TaskFormField::Status => form.cycle_status(),
                TaskFormField::Kind => form.cycle_kind(),
                TaskFormField::Size => form.cycle_size(),
                _ => {}
            }
        }

        // Text input for text fields
        KeyCode::Char(c) => {
            use crate::ui::components::TaskFormField;
            match form.current_field {
                TaskFormField::Title => form.title_input.push(c),
                TaskFormField::Notes => form.notes_input.push(c),
                TaskFormField::DueDate => form.due_date_input.push(c),
                _ => {}
            }
        }
        KeyCode::Backspace => {
            use crate::ui::components::TaskFormField;
            match form.current_field {
                TaskFormField::Title => { form.title_input.pop(); }
                TaskFormField::Notes => { form.notes_input.pop(); }
                TaskFormField::DueDate => { form.due_date_input.pop(); }
                _ => {}
            }
        }

        _ => {}
    }
}

fn handle_confirm_mode(app: &mut App, key: KeyEvent) {
    let Some(ref mut modal) = app.confirm_modal else {
        app.mode = AppMode::Normal;
        return;
    };

    match key.code {
        KeyCode::Esc | KeyCode::Char('n') => {
            app.confirm_modal = None;
            app.mode = AppMode::Normal;
        }
        KeyCode::Enter | KeyCode::Char('y') => {
            if modal.selected {
                app.execute_confirm();
            } else {
                app.confirm_modal = None;
                app.mode = AppMode::Normal;
            }
        }
        KeyCode::Tab | KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
            modal.toggle();
        }
        _ => {}
    }
}

fn handle_settings_mode(app: &mut App, key: KeyEvent) {
    let editing = app.settings_view.editing;

    if editing {
        match key.code {
            KeyCode::Esc => app.settings_view.cancel_editing(),
            KeyCode::Enter => {
                app.settings_view.save_field();
                // Save config to disk
                if let Err(e) = app.settings_view.config.save() {
                    app.show_error(format!("Failed to save config: {}", e));
                } else {
                    app.config = app.settings_view.config.clone();
                    app.settings_view.saved_message = Some("Saved!".to_string());
                }
            }
            KeyCode::Backspace => app.settings_view.input.backspace(),
            KeyCode::Delete => app.settings_view.input.delete(),
            KeyCode::Left => app.settings_view.input.move_left(),
            KeyCode::Right => app.settings_view.input.move_right(),
            KeyCode::Home => app.settings_view.input.move_start(),
            KeyCode::End => app.settings_view.input.move_end(),
            KeyCode::Char(c) => app.settings_view.input.insert(c),
            _ => {}
        }
    } else {
        match key.code {
            // View switching with number keys
            KeyCode::Char('1') => app.switch_to_view(SidebarItem::Inbox),
            KeyCode::Char('2') => app.switch_to_view(SidebarItem::Today),
            KeyCode::Char('3') => app.switch_to_view(SidebarItem::Upcoming),
            KeyCode::Char('4') => app.switch_to_view(SidebarItem::Anytime),
            KeyCode::Char('5') => app.switch_to_view(SidebarItem::Completed),
            KeyCode::Char('6') => app.switch_to_view(SidebarItem::Review),
            KeyCode::Char('7') => app.switch_to_view(SidebarItem::GitHub),
            KeyCode::Char('8') => app.switch_to_view(SidebarItem::Toggl),
            // 9 is current view (Settings), no need to switch

            KeyCode::Char('j') | KeyCode::Down => app.settings_view.next_field(),
            KeyCode::Char('k') | KeyCode::Up => app.settings_view.prev_field(),
            KeyCode::Enter | KeyCode::Char('e') => app.settings_view.start_editing(),
            KeyCode::Char('s') => {
                if let Err(e) = app.settings_view.config.save() {
                    app.show_error(format!("Failed to save config: {}", e));
                } else {
                    app.config = app.settings_view.config.clone();
                    app.settings_view.saved_message = Some("Config saved!".to_string());
                }
            }
            // Navigation - allow leaving settings
            KeyCode::Tab => {
                app.cycle_focus();
                app.mode = AppMode::Normal;
            }
            KeyCode::BackTab => {
                app.cycle_focus_reverse();
                app.mode = AppMode::Normal;
            }
            KeyCode::Char('h') | KeyCode::Left => {
                app.focus = FocusArea::Sidebar;
                app.mode = AppMode::Normal;
            }
            _ => {}
        }
    }
}
