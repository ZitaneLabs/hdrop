import styled from 'styled-components'
import { createBrowserRouter, RouterProvider } from 'react-router-dom'
import { RecoilRoot } from 'recoil'

import HomeView from './views/HomeView'
import FileDetailView from './views/FileDetailView'
import Logo from './components/Logo'
import SecurityView from './views/SecurityView'

const router = createBrowserRouter([
  {
    path: '/',
    element: <HomeView />,
  },
  {
    path: '/:accessToken',
    element: <FileDetailView />,
    caseSensitive: true,
  },
  {
    path: '/security',
    element: <SecurityView />,
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
`
