import styled from 'styled-components'
import Header from '../components/Header'
import Footer from '../components/Footer'

const View = ({ className, children }) => {
    return (
        <div className={className}>
            <Header />
            <main>
                {children}
            </main>
            <Footer />
        </div>
    )
}

export default styled(View)`
    display: grid;
    grid-template-columns: 1fr;
    grid-template-rows: auto 1fr auto;
    grid-template-areas: 'header' 'main' 'footer';

    width: 100%;
    height: 100%;

    & > header {
        grid-area: 'header';
    }

    & > main {
        grid-area: 'main';
        width: 100%;
        height: 100%;
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        overflow: auto;
    }

    & > footer {
        grid-area: 'footer';
    }
`