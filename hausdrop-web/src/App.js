import styled from 'styled-components'
import { createBrowserRouter, RouterProvider } from 'react-router-dom'
import { RecoilRoot } from 'recoil'

import HomeView from './views/HomeView'
import FileDetailView from './views/FileDetailView'
import Logo from './components/Logo'

const router = createBrowserRouter([
  {
    path: '/',
    element: <HomeView />,
  },
  {
    path: '/:accessToken',
    element: <FileDetailView />,
  }
])

const App = ({ className }) => {
  return (
    <RecoilRoot>
      <div className={className}>
        <RouterProvider router={router} />
      </div>
    </RecoilRoot>
  )
}

export default styled(App)`
  margin: 0;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
`
