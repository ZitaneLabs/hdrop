import styled from 'styled-components'

/**
 * A row of status bubbles
 * 
 * @param {{
 * className: string,
 * children: React.ReactNode
 * }} props
 */
const StatusBubbleRow = ({ className, children }) => {
    return (
        <div className={className}>
            {children}
        </div>
    )
}

export default styled(StatusBubbleRow)`
    width: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    transition: gap .25s ease;
    gap: 2rem;

    @media screen and (min-width: 700px) {
        gap: 3rem;
    }

    @media screen and (min-width: 1200px) {
        gap: 4rem;
    }
`