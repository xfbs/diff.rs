use super::*;

#[function_component]
pub fn NotFound() -> Html {
    html! {
        <>
            <SimpleNavbar />
            <Content>
                <Error title={"Not found"} status={"The URL was not found"} />
            </Content>
        </>
    }
}
