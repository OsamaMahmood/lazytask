// View switching logic

pub enum View {
    TaskList,
    TaskDetail,
    Reports,
    Settings,
    Help,
}

pub struct NavigationHandler {
    current_view: View,
    view_stack: Vec<View>,
}

impl NavigationHandler {
    pub fn new() -> Self {
        NavigationHandler {
            current_view: View::TaskList,
            view_stack: Vec::new(),
        }
    }

    pub fn navigate_to(&mut self, view: View) {
        self.view_stack.push(std::mem::replace(&mut self.current_view, view));
    }

    pub fn go_back(&mut self) -> bool {
        if let Some(previous_view) = self.view_stack.pop() {
            self.current_view = previous_view;
            true
        } else {
            false
        }
    }

    pub fn current_view(&self) -> &View {
        &self.current_view
    }
}
