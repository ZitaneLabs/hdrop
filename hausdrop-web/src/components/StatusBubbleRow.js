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
    gap: 5rem;
`