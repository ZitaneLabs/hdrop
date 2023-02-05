import styled from "styled-components"

import Logo from "./Logo"

const Header = ({ className }) => {
    return (
        <header className={className}>
            <Logo />
        </header>
    )
}

export default styled(Header)`
    width: 100%;
    height: 4rem;

    display: flex;
    justify-content: center;
    align-items: flex-end;

    @media screen and (min-width: 700px) and (min-height: 700px) {
        height: 4rem;
    }
`