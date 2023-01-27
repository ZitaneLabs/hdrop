import styled from 'styled-components'

const UploadProgressBar = ({ className, progress }) => {
    return (
        <div className={className}>
            <div className="inner" style={{ width: `${progress * 100}%` }} />
        </div>
    )
}

export default styled(UploadProgressBar)`
    width: 100%;
    height: 1rem;
    background-color: hsl(0,0%,15%);
    border-radius: 0.5rem;

    & .inner {
        height: 100%;
        background-color: hsl(0,0%,90%);
        border-radius: 0.5rem;
        transition: width 0.2s ease;
    }
`