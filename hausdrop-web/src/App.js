import styled from 'styled-components'
import { createBrowserRouter, RouterProvider } from 'react-router-dom'
import { RecoilRoot } from 'recoil'

import HomeView from './views/HomeView'
import FileDetailView from './views/FileDetailView'

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
    <div className={className}>
      <RecoilRoot>
        <RouterProvider router={router} />
      </RecoilRoot>
    </div>
  )
}

export default styled(App)`
  margin: 0;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
`
