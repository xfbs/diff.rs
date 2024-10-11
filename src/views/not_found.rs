use super::*;

/// Not found view, shows generic error.
#[function_component]
pub fn NotFound() -> Html {
    html! {
        <div class="flex flex-col min-h-screen">
            <div class="flex-1">
                <SimpleNavbar />
                <Content>
                    <Center>
                        <Error title={"Not found"} status={"The URL was not found"} />
                    </Center>
                </Content>
            </div>
            <Footer />
        </div>
    }
}
