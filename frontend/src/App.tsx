import '@mantine/core/styles.css';
import { AppShell, MantineProvider } from '@mantine/core';
import { BrowserRouter, Route, Routes } from 'react-router-dom';
import { Nav } from './components/Nav';
import { About } from './pages/About';
import { Education } from './pages/Education';
import { Experience } from './pages/Experience';
import { PipelineHealth } from './pages/PipelineHealth';
import { Skills } from './pages/Skills';
import { BP, theme } from './theme';

export function App() {
  return (
    <MantineProvider theme={theme} forceColorScheme="dark">
      <BrowserRouter>
        <AppShell
          navbar={{ width: 280, breakpoint: 'sm' }}
          padding={0}
        >
          <AppShell.Navbar
            style={{
              background: BP.surfaceNav,
              borderRight: `1px solid ${BP.border}`,
            }}
          >
            <Nav />
          </AppShell.Navbar>

          <AppShell.Main style={{ background: BP.bgMain, minHeight: '100vh' }}>
            <Routes>
              <Route path="/"           element={<About />} />
              <Route path="/experience" element={<Experience />} />
              <Route path="/skills"     element={<Skills />} />
              <Route path="/education"  element={<Education />} />
              <Route path="/pipeline"   element={<PipelineHealth />} />
            </Routes>
          </AppShell.Main>
        </AppShell>
      </BrowserRouter>
    </MantineProvider>
  );
}
