import React, { useState } from 'react';
import { ScrollView, Alert } from 'react-native';
import {
  YStack,
  XStack,
  Card,
  H1,
  H2,
  H3,
  Paragraph,
  Button,
  Switch,
  Input,
  Separator,
  Select,
  Badge,
} from '@tamagui/core';
import { 
  Settings, 
  User, 
  Bell, 
  Shield, 
  Globe, 
  Palette, 
  Accessibility,
  HelpCircle,
  LogOut,
  ChevronDown
} from '@tamagui/lucide-icons';

export default function SettingsScreen() {
  const [notifications, setNotifications] = useState({
    proposals: true,
    votes: true,
    discussions: false,
    community: true,
    email: false,
  });

  const [accessibility, setAccessibility] = useState({
    highContrast: false,
    largeText: false,
    reducedMotion: false,
    screenReader: false,
  });

  const [language, setLanguage] = useState('en');
  const [theme, setTheme] = useState('auto');

  const handleNotificationChange = (key: keyof typeof notifications) => {
    setNotifications(prev => ({ ...prev, [key]: !prev[key] }));
  };

  const handleAccessibilityChange = (key: keyof typeof accessibility) => {
    setAccessibility(prev => ({ ...prev, [key]: !prev[key] }));
  };

  const handleLogout = () => {
    Alert.alert(
      'Logout',
      'Are you sure you want to logout?',
      [
        { text: 'Cancel', style: 'cancel' },
        { text: 'Logout', style: 'destructive', onPress: () => {
          // Handle logout logic here
          Alert.alert('Success', 'You have been logged out.');
        }},
      ]
    );
  };

  return (
    <ScrollView style={{ flex: 1, backgroundColor: '#f8fafc' }}>
      <YStack padding="$4" space="$4">
        {/* Header */}
        <YStack space="$2">
          <H1 color="$primary">Settings</H1>
          <Paragraph color="$gray11">Customize your AgoraNet experience</Paragraph>
        </YStack>

        {/* Profile Settings */}
        <Card padding="$4">
          <YStack space="$4">
            <XStack alignItems="center" space="$2">
              <User size={20} color="$blue9" />
              <H2>Profile</H2>
            </XStack>
            
            <YStack space="$3">
              <YStack space="$2">
                <Paragraph fontWeight="600">Display Name</Paragraph>
                <Input defaultValue="Demo User" placeholder="Enter your name" />
              </YStack>
              
              <YStack space="$2">
                <Paragraph fontWeight="600">Email Address</Paragraph>
                <Input defaultValue="user@example.com" placeholder="Enter your email" />
              </YStack>

              <YStack space="$2">
                <Paragraph fontWeight="600">Bio</Paragraph>
                <Input 
                  defaultValue="Cooperative member interested in democratic governance" 
                  placeholder="Tell the community about yourself"
                  multiline
                />
              </YStack>

              <Button theme="blue" size="$3">
                Update Profile
              </Button>
            </YStack>
          </YStack>
        </Card>

        {/* Notification Settings */}
        <Card padding="$4">
          <YStack space="$4">
            <XStack alignItems="center" space="$2">
              <Bell size={20} color="$orange9" />
              <H2>Notifications</H2>
            </XStack>
            
            <YStack space="$3">
              <XStack justifyContent="space-between" alignItems="center">
                <YStack flex={1}>
                  <Paragraph fontWeight="600">New Proposals</Paragraph>
                  <Paragraph size="$3" color="$gray11">Get notified when new proposals are created</Paragraph>
                </YStack>
                <Switch 
                  checked={notifications.proposals}
                  onCheckedChange={() => handleNotificationChange('proposals')}
                />
              </XStack>

              <XStack justifyContent="space-between" alignItems="center">
                <YStack flex={1}>
                  <Paragraph fontWeight="600">Voting Reminders</Paragraph>
                  <Paragraph size="$3" color="$gray11">Reminders before voting deadlines</Paragraph>
                </YStack>
                <Switch 
                  checked={notifications.votes}
                  onCheckedChange={() => handleNotificationChange('votes')}
                />
              </XStack>

              <XStack justifyContent="space-between" alignItems="center">
                <YStack flex={1}>
                  <Paragraph fontWeight="600">Discussion Mentions</Paragraph>
                  <Paragraph size="$3" color="$gray11">When someone mentions you in discussions</Paragraph>
                </YStack>
                <Switch 
                  checked={notifications.discussions}
                  onCheckedChange={() => handleNotificationChange('discussions')}
                />
              </XStack>

              <XStack justifyContent="space-between" alignItems="center">
                <YStack flex={1}>
                  <Paragraph fontWeight="600">Community Updates</Paragraph>
                  <Paragraph size="$3" color="$gray11">News and updates from the community</Paragraph>
                </YStack>
                <Switch 
                  checked={notifications.community}
                  onCheckedChange={() => handleNotificationChange('community')}
                />
              </XStack>

              <XStack justifyContent="space-between" alignItems="center">
                <YStack flex={1}>
                  <Paragraph fontWeight="600">Email Notifications</Paragraph>
                  <Paragraph size="$3" color="$gray11">Receive notifications via email</Paragraph>
                </YStack>
                <Switch 
                  checked={notifications.email}
                  onCheckedChange={() => handleNotificationChange('email')}
                />
              </XStack>
            </YStack>
          </YStack>
        </Card>

        {/* Appearance Settings */}
        <Card padding="$4">
          <YStack space="$4">
            <XStack alignItems="center" space="$2">
              <Palette size={20} color="$purple9" />
              <H2>Appearance</H2>
            </XStack>
            
            <YStack space="$3">
              <YStack space="$2">
                <Paragraph fontWeight="600">Theme</Paragraph>
                <Select value={theme} onValueChange={setTheme}>
                  <Select.Trigger>
                    <Select.Value placeholder="Select theme" />
                    <ChevronDown size={16} />
                  </Select.Trigger>
                  <Select.Content>
                    <Select.Item index={0} value="light">
                      <Select.ItemText>Light</Select.ItemText>
                    </Select.Item>
                    <Select.Item index={1} value="dark">
                      <Select.ItemText>Dark</Select.ItemText>
                    </Select.Item>
                    <Select.Item index={2} value="auto">
                      <Select.ItemText>Auto (System)</Select.ItemText>
                    </Select.Item>
                  </Select.Content>
                </Select>
              </YStack>

              <YStack space="$2">
                <Paragraph fontWeight="600">Language</Paragraph>
                <Select value={language} onValueChange={setLanguage}>
                  <Select.Trigger>
                    <Select.Value placeholder="Select language" />
                    <ChevronDown size={16} />
                  </Select.Trigger>
                  <Select.Content>
                    <Select.Item index={0} value="en">
                      <Select.ItemText>English</Select.ItemText>
                    </Select.Item>
                    <Select.Item index={1} value="es">
                      <Select.ItemText>Español</Select.ItemText>
                    </Select.Item>
                    <Select.Item index={2} value="fr">
                      <Select.ItemText>Français</Select.ItemText>
                    </Select.Item>
                    <Select.Item index={3} value="de">
                      <Select.ItemText>Deutsch</Select.ItemText>
                    </Select.Item>
                  </Select.Content>
                </Select>
              </YStack>
            </YStack>
          </YStack>
        </Card>

        {/* Accessibility Settings */}
        <Card padding="$4">
          <YStack space="$4">
            <XStack alignItems="center" space="$2">
              <Accessibility size={20} color="$green9" />
              <H2>Accessibility</H2>
            </XStack>
            
            <YStack space="$3">
              <XStack justifyContent="space-between" alignItems="center">
                <YStack flex={1}>
                  <Paragraph fontWeight="600">High Contrast Mode</Paragraph>
                  <Paragraph size="$3" color="$gray11">Increase contrast for better visibility</Paragraph>
                </YStack>
                <Switch 
                  checked={accessibility.highContrast}
                  onCheckedChange={() => handleAccessibilityChange('highContrast')}
                />
              </XStack>

              <XStack justifyContent="space-between" alignItems="center">
                <YStack flex={1}>
                  <Paragraph fontWeight="600">Large Text</Paragraph>
                  <Paragraph size="$3" color="$gray11">Increase text size throughout the app</Paragraph>
                </YStack>
                <Switch 
                  checked={accessibility.largeText}
                  onCheckedChange={() => handleAccessibilityChange('largeText')}
                />
              </XStack>

              <XStack justifyContent="space-between" alignItems="center">
                <YStack flex={1}>
                  <Paragraph fontWeight="600">Reduced Motion</Paragraph>
                  <Paragraph size="$3" color="$gray11">Minimize animations and transitions</Paragraph>
                </YStack>
                <Switch 
                  checked={accessibility.reducedMotion}
                  onCheckedChange={() => handleAccessibilityChange('reducedMotion')}
                />
              </XStack>

              <XStack justifyContent="space-between" alignItems="center">
                <YStack flex={1}>
                  <Paragraph fontWeight="600">Screen Reader Support</Paragraph>
                  <Paragraph size="$3" color="$gray11">Optimize for screen reader navigation</Paragraph>
                </YStack>
                <Switch 
                  checked={accessibility.screenReader}
                  onCheckedChange={() => handleAccessibilityChange('screenReader')}
                />
              </XStack>
            </YStack>
          </YStack>
        </Card>

        {/* Privacy & Security */}
        <Card padding="$4">
          <YStack space="$4">
            <XStack alignItems="center" space="$2">
              <Shield size={20} color="$red9" />
              <H2>Privacy & Security</H2>
            </XStack>
            
            <YStack space="$3">
              <Button theme="gray" justifyContent="flex-start">
                <XStack alignItems="center" space="$2" flex={1}>
                  <Paragraph flex={1}>Data Export</Paragraph>
                  <Paragraph size="$3" color="$gray11">Download your data</Paragraph>
                </XStack>
              </Button>

              <Button theme="gray" justifyContent="flex-start">
                <XStack alignItems="center" space="$2" flex={1}>
                  <Paragraph flex={1}>Privacy Policy</Paragraph>
                  <Paragraph size="$3" color="$gray11">View our privacy policy</Paragraph>
                </XStack>
              </Button>

              <Button theme="gray" justifyContent="flex-start">
                <XStack alignItems="center" space="$2" flex={1}>
                  <Paragraph flex={1}>Terms of Service</Paragraph>
                  <Paragraph size="$3" color="$gray11">Read terms and conditions</Paragraph>
                </XStack>
              </Button>
            </YStack>
          </YStack>
        </Card>

        {/* Help & Support */}
        <Card padding="$4">
          <YStack space="$4">
            <XStack alignItems="center" space="$2">
              <HelpCircle size={20} color="$blue9" />
              <H2>Help & Support</H2>
            </XStack>
            
            <YStack space="$3">
              <Button theme="gray" justifyContent="flex-start">
                <XStack alignItems="center" space="$2" flex={1}>
                  <Paragraph flex={1}>User Guide</Paragraph>
                  <Paragraph size="$3" color="$gray11">Learn how to use AgoraNet</Paragraph>
                </XStack>
              </Button>

              <Button theme="gray" justifyContent="flex-start">
                <XStack alignItems="center" space="$2" flex={1}>
                  <Paragraph flex={1}>Contact Support</Paragraph>
                  <Paragraph size="$3" color="$gray11">Get help from our team</Paragraph>
                </XStack>
              </Button>

              <Button theme="gray" justifyContent="flex-start">
                <XStack alignItems="center" space="$2" flex={1}>
                  <Paragraph flex={1}>Report a Bug</Paragraph>
                  <Paragraph size="$3" color="$gray11">Help us improve AgoraNet</Paragraph>
                </XStack>
              </Button>
            </YStack>
          </YStack>
        </Card>

        {/* About */}
        <Card padding="$4" backgroundColor="$gray1">
          <YStack space="$3" alignItems="center">
            <H3>AgoraNet v0.1.0</H3>
            <Paragraph color="$gray11" textAlign="center">
              Built for the InterCooperative Network with ❤️ for democratic governance
            </Paragraph>
            <XStack space="$2">
              <Badge theme="blue">React Native</Badge>
              <Badge theme="purple">Tamagui</Badge>
              <Badge theme="green">Expo</Badge>
            </XStack>
          </YStack>
        </Card>

        {/* Logout */}
        <Button
          theme="red"
          icon={LogOut}
          onPress={handleLogout}
          marginBottom="$4"
        >
          Logout
        </Button>
      </YStack>
    </ScrollView>
  );
}