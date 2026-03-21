use crate::{
    Quartz,
    endpoint::{error::EndpointError, handle::EndpointHandle},
    error::Result,
    state::field::StateField,
};

pub mod value;
pub mod error;
pub mod handle;
pub mod resolved_endpoint;

#[cfg(test)]
mod tests;

pub struct EndpointManager<'a> {
    quartz: &'a Quartz,
}

impl<'a> EndpointManager<'a> {
    pub fn new(quartz: &'a Quartz) -> Self {
        Self { quartz }
    }
    pub fn new_handle(&self, name: &str) -> EndpointHandle<'_> {
        EndpointHandle::new(self.quartz, name.into())
    }

    pub fn current(&self) -> Option<EndpointHandle<'_>> {
        let curr_endpoint_name = self.quartz.state().get(StateField::Endpoint).ok();

        curr_endpoint_name.map(|handle_name| EndpointHandle::new(self.quartz, handle_name.into()))
    }

    pub fn switch(&self, mut handle: String) -> Result<EndpointHandle<'_>> {
        if handle == "-" {
            let previous_handle = self.quartz.state().get(StateField::PreviousEndpoint)?;
            handle = previous_handle;
        }

        let handle = EndpointHandle::new(self.quartz, handle.into());

        if !handle.exists() {
            return Err(EndpointError::HandleNotFound(handle.head()).into());
        }

        let previous = self.quartz.state().get(StateField::Endpoint);
        self.quartz
            .state()
            .set(StateField::Endpoint, &handle.path.join("/"))?;

        if let Ok(prev) = previous {
            self.quartz
                .state()
                .set(StateField::PreviousEndpoint, &prev)?;
        }

        Ok(handle)
    }

    pub fn copy(&self, recursive: bool, src: &str, dest: &str) -> Result {
        let src_handle = EndpointHandle::new(self.quartz, src.into());
        if !src_handle.exists() {
            return Err(EndpointError::HandleNotFound(src.to_owned()).into());
        }
        let dest_handle = EndpointHandle::new(self.quartz, dest.into());
        dest_handle.ensure_dir()?;
        if let Ok(endpoint) = src_handle.endpoint() {
            dest_handle.write_endpoint(&endpoint)?;
        }

        if recursive {
            for child in src_handle.children()? {
                let child_name = child.handle();
                let mut new_handle = EndpointHandle::new(self.quartz, child.path);

                // Replace original prefix with the dest one
                let dest_handle_prefix = dest_handle.path[0].clone();
                let _ = std::mem::replace(&mut new_handle.path[0], dest_handle_prefix);

                self.copy(true, &child_name, &new_handle.handle())?;
            }
        }

        Ok(())
    }

    pub fn mv(&self, src: &str, dest: &str) -> Result {
        let src_handle = EndpointHandle::new(self.quartz, src.into());
        if !src_handle.exists() {
            return Err(EndpointError::HandleNotFound(src.to_owned()).into());
        }

        // TODO: this might be one of the lazyest solutions so far
        self.copy(true, src, dest)?;
        src_handle.delete(true)?;

        Ok(())
    }
}
